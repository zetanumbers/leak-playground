use std::{marker::PhantomPinned, pin::Pin, ptr};

use crate::{sync_queue, util, SyncQueue, ThreadPool};

pub struct Executor<'f, F> {
    // drops first
    sender: sync_queue::spmc::Sender<F, SyncQueue<F>>,
    threads: Option<ThreadPool<'f>>,
    _pinned: PhantomPinned,
}

impl<F> Executor<'_, F> {
    /// You have to use [`Executor::start`] to start execution
    pub const fn unresumed() -> Self {
        Executor {
            sender: sync_queue::spmc::Sender::from_queue(SyncQueue::new()),
            threads: None,
            _pinned: PhantomPinned,
        }
    }
}

impl<'f, F> Executor<'f, F>
where
    F: FnOnce() + Send + 'f,
{
    pub fn start(self: Pin<&mut Self>) {
        if self.threads.is_some() {
            return;
        };

        unsafe {
            let this = self.get_unchecked_mut();

            let queue = util::UnsafeInternalRef::new(sync_queue::spmc::Sender::queue_raw(
                ptr::addr_of!(this.sender),
            ));
            let receiver = sync_queue::spmc::Receiver::from_queue(queue);
            this.threads = Some(ThreadPool::from_jobs_iter(receiver))
        }
    }

    // TODO: support Arc somehow?
    pub fn sender(&self) -> &sync_queue::spmc::Sender<F, SyncQueue<F>> {
        &self.sender
    }
}
