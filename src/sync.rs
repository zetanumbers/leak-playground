pub mod mpsc {
    use std::sync::mpsc;

    pub fn rendezvous_channel<T>() -> (mpsc::SyncSender<T>, mpsc::Receiver<T>) {
        mpsc::sync_channel(0)
    }
}
