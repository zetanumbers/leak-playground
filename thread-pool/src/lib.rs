use std::{marker::PhantomPinned, num::NonZeroUsize, pin::Pin, ptr};

pub use sync_queue::SyncQueue;

mod thread {
    pub use leak_playground_std::thread::*;
    pub use std::thread::*;
}

pub mod sync_queue;
mod util;

const DEFAULT_NUM_THREADS: usize = 4;

pub struct ThreadPool<'queue> {
    threads: Vec<thread::JoinGuard<'queue, ()>>,
}

impl<'queue> ThreadPool<'queue> {
    pub fn from_jobs_iter<Q>(queue: Q) -> Self
    where
        Q: IntoIterator + Clone + Send + 'queue,
        Q::Item: FnOnce(),
    {
        let num_threads = thread::available_parallelism()
            .ok()
            .or_else(|| NonZeroUsize::new(DEFAULT_NUM_THREADS))
            .unwrap()
            .get();

        let mut threads = Vec::with_capacity(num_threads);
        threads.resize_with(num_threads, || {
            let queue = queue.clone();
            thread::spawn_scoped(move || {
                queue.into_iter().for_each(|job| job());
            })
        });
        ThreadPool { threads }
    }

    pub fn is_finished(&self) -> bool {
        self.threads.iter().all(|t| t.is_finished())
    }
}

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
