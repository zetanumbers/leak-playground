use std::num::NonZeroUsize;

use leak_playground_std::thread;

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
        let num_threads = std::thread::available_parallelism()
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
