use std::collections::VecDeque;
use std::sync::Condvar;
use std::sync::Mutex;

pub mod spmc;

#[derive(Default)]
pub struct SyncQueue<T> {
    items: Mutex<Option<VecDeque<T>>>,
    cond_var: Condvar,
}

impl<T> SyncQueue<T> {
    pub const fn new() -> Self {
        SyncQueue {
            items: Mutex::new(Some(VecDeque::new())),
            cond_var: Condvar::new(),
        }
    }

    // TODO: push_bounded
    pub fn push(&self, item: T) -> Result<(), SyncQueuePushError<T>> {
        let mut lock = self.items.lock().expect("job queue is poisoned");
        match &mut *lock {
            Some(queue) => {
                queue.push_back(item);
                Ok(())
            }
            None => Err(SyncQueuePushError {
                source: ClosedSyncQueueError(()),
                item,
            }),
        }
    }

    pub fn pop(&self) -> Result<T, ClosedSyncQueueError> {
        let mut res_lock = self.items.lock();
        loop {
            let mut lock = res_lock.expect("job queue is poisoned");
            let queue = lock.as_mut().ok_or(ClosedSyncQueueError(()))?;
            if let Some(item) = queue.pop_front() {
                if !queue.is_empty() {
                    self.cond_var.notify_one();
                }
                return Ok(item);
            } else {
                res_lock = self.cond_var.wait(lock);
            };
        }
    }

    pub fn close(&self) -> Result<VecDeque<T>, ClosedSyncQueueError> {
        let rest = self.items.lock().expect("job queue is poisoned").take();
        self.cond_var.notify_all();
        rest.ok_or(ClosedSyncQueueError(()))
    }

    pub fn is_closed(&self) -> bool {
        self.items.lock().expect("job queue is poisoned").is_none()
    }

    pub fn pop_iter(&self) -> PopIter<'_, T> {
        spmc::Receiver::from(self).into_iter()
    }
}

impl<T> From<VecDeque<T>> for SyncQueue<T> {
    fn from(value: VecDeque<T>) -> Self {
        SyncQueue {
            items: Mutex::new(Some(value)),
            cond_var: Condvar::new(),
        }
    }
}

impl<T> AsRef<SyncQueue<T>> for SyncQueue<T> {
    fn as_ref(&self) -> &SyncQueue<T> {
        self
    }
}

impl<'a, T> IntoIterator for &'a SyncQueue<T> {
    type Item = T;
    type IntoIter = PopIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.pop_iter()
    }
}

impl<T> IntoIterator for SyncQueue<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        spmc::Receiver::from(self).into_iter()
    }
}

pub type PopIter<'a, T> = spmc::ReceiverIter<'a, T>;
pub type IntoIter<T> = spmc::ReceiverIntoIter<T, SyncQueue<T>>;

#[derive(Debug)]
pub struct ClosedSyncQueueError(());

impl std::fmt::Display for ClosedSyncQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "sync queue has been closed".fmt(f)
    }
}

impl std::error::Error for ClosedSyncQueueError {}

pub struct SyncQueuePushError<T> {
    source: ClosedSyncQueueError,
    item: T,
}

impl<T> SyncQueuePushError<T> {
    pub fn into_item(self) -> T {
        self.item
    }
}

impl<T> From<SyncQueuePushError<T>> for ClosedSyncQueueError {
    fn from(value: SyncQueuePushError<T>) -> Self {
        value.source
    }
}

impl<T> std::fmt::Display for SyncQueuePushError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "sending through sync queue failed".fmt(f)
    }
}

impl<T> std::fmt::Debug for SyncQueuePushError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncQueuePushError")
            .field("source", &self.source)
            .field("item", &format_args!("<...>"))
            .finish()
    }
}

impl<T> std::error::Error for SyncQueuePushError<T> {}
