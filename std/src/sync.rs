//! Possible [`std::sync`] additions and replacements.

mod arc;

pub mod mpsc {
    use std::sync::mpsc;

    use crate::marker::Forget;

    pub fn rendezvous_channel<T>() -> (mpsc::SyncSender<T>, mpsc::Receiver<T>) {
        mpsc::sync_channel(0)
    }

    pub fn sync_channel<T: Forget>(bound: usize) -> (mpsc::SyncSender<T>, mpsc::Receiver<T>) {
        mpsc::sync_channel(bound)
    }

    pub fn channel<T: Forget>() -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        mpsc::channel()
    }

    pub unsafe fn sync_channel_unchecked<T>(
        bound: usize,
    ) -> (mpsc::SyncSender<T>, mpsc::Receiver<T>) {
        mpsc::sync_channel(bound)
    }

    pub unsafe fn channel_unchecked<T>() -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        mpsc::channel()
    }
}

pub use arc::*;
