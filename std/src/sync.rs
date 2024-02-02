//! Possible [`std::sync`] additions and edits.

mod arc;

pub mod mpsc {
    use std::sync::mpsc;

    use crate::marker::Leak;

    pub fn rendezvous_channel<T>() -> (mpsc::SyncSender<T>, mpsc::Receiver<T>) {
        mpsc::sync_channel(0)
    }

    pub fn sync_channel<T: Leak>(bound: usize) -> (mpsc::SyncSender<T>, mpsc::Receiver<T>) {
        mpsc::sync_channel(bound)
    }

    pub fn channel<T: Leak>() -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
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
