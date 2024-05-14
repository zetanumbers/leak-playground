use std::{marker::PhantomData, ptr};

use super::SyncQueue;

// TODO: bound
pub fn unbound<T, Q>(queue: Q) -> (Sender<T, Q>, Receiver<T, Q>)
where
    Q: AsRef<SyncQueue<T>> + Clone,
{
    (queue.clone().into(), queue.into())
}

/// Closes the queue on drop.
pub struct Sender<T, Q>
where
    Q: AsRef<SyncQueue<T>>,
{
    _marker: PhantomData<fn(T)>,
    queue: Q,
}

impl<T, Q> Sender<T, Q>
where
    Q: AsRef<SyncQueue<T>>,
{
    pub const fn from_queue(queue: Q) -> Self {
        Sender {
            _marker: PhantomData,
            queue,
        }
    }
}

impl<T, Q> From<Q> for Sender<T, Q>
where
    Q: AsRef<SyncQueue<T>>,
{
    fn from(queue: Q) -> Self {
        Sender::from_queue(queue)
    }
}

impl<T, Q> Drop for Sender<T, Q>
where
    Q: AsRef<SyncQueue<T>>,
{
    fn drop(&mut self) {
        // Error means user already closed the queue
        let _ = self.queue.as_ref().close();
    }
}

impl<T, Q> Sender<T, Q>
where
    Q: AsRef<SyncQueue<T>>,
{
    pub fn send(&self, item: T) -> Result<(), super::SyncQueuePushError<T>> {
        self.queue.as_ref().push(item)
    }

    pub fn queue(&self) -> &Q {
        &self.queue
    }

    /// Get a pointer to the queue
    ///
    /// # Safety
    ///
    /// `this` must point to a valid [`Receiver`]
    pub unsafe fn queue_raw(this: *const Self) -> *const Q {
        ptr::addr_of!((*this).queue)
    }
}

pub struct Receiver<T, Q> {
    _marker: PhantomData<fn() -> T>,
    queue: Q,
}

impl<T, Q> From<Q> for Receiver<T, Q> {
    fn from(queue: Q) -> Self {
        Receiver::from_queue(queue)
    }
}

impl<T, Q> Receiver<T, Q> {
    pub const fn from_queue(queue: Q) -> Self {
        Self {
            _marker: PhantomData,
            queue,
        }
    }

    pub fn recv(&self) -> Result<T, super::ClosedSyncQueueError>
    where
        Q: AsRef<SyncQueue<T>>,
    {
        self.queue.as_ref().pop()
    }

    pub fn iter(&self) -> ReceiverIter<'_, T>
    where
        Q: AsRef<SyncQueue<T>>,
    {
        Receiver {
            queue: self.queue.as_ref(),
            _marker: PhantomData,
        }
        .into_iter()
    }

    pub fn queue(&self) -> &Q {
        &self.queue
    }

    /// Get a pointer to the queue
    ///
    /// # Safety
    ///
    /// `this` must point to a valid [`Receiver`]
    pub unsafe fn queue_raw(this: *const Self) -> *const Q {
        ptr::addr_of!((*this).queue)
    }
}

impl<T, Q: Clone> Clone for Receiver<T, Q> {
    fn clone(&self) -> Self {
        Receiver {
            _marker: PhantomData,
            queue: self.queue.clone(),
        }
    }
}

pub type ReceiverIter<'a, T> = ReceiverIntoIter<T, &'a SyncQueue<T>>;

impl<T, Q> IntoIterator for Receiver<T, Q>
where
    Q: AsRef<SyncQueue<T>>,
{
    type Item = T;
    type IntoIter = ReceiverIntoIter<T, Q>;

    fn into_iter(self) -> Self::IntoIter {
        ReceiverIntoIter { inner: self }
    }
}

impl<'a, T, Q> IntoIterator for &'a Receiver<T, Q>
where
    Q: AsRef<SyncQueue<T>>,
{
    type Item = T;
    type IntoIter = ReceiverIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct ReceiverIntoIter<T, Q> {
    inner: Receiver<T, Q>,
}

impl<T, Q> Iterator for ReceiverIntoIter<T, Q>
where
    Q: AsRef<SyncQueue<T>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.recv().ok()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.inner.queue.as_ref().is_closed().then_some(0))
    }
}
