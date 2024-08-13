use std::sync::Barrier;

use leak_playground_std::sync::{mpsc::rendezvous_channel, Arc};

fn main() {
    let mut s = "Hello world".to_owned();
    let barrier = Arc::new(Barrier::new(2));
    {
        let (tx, rx) = rendezvous_channel();
        let barrier = Arc::clone(&barrier);
        let s = s.as_str();

        let task = leak_playground_std::thread::spawn_scoped(move || {
            let _this_task = rx.recv().unwrap();
            barrier.wait();
            dbg!(s);
            barrier.wait();
        });
        tx.send(task).unwrap();
    }
    s.clear();
    s.push_str("Sad");
    barrier.wait();
    barrier.wait();
}
