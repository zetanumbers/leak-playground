use std::sync::{Barrier, Mutex};

use leak_playground_std::sync::Arc;

fn main() {
    let mut s = "Hello world".to_owned();
    let barrier = Arc::new(Barrier::new(2));
    {
        let rendezvous = Mutex::new(None);
        let rendezvous = &rendezvous;
        let barrier = Arc::clone(&barrier);
        let s = s.as_str();

        let task = leak_playground_std::thread::spawn_scoped(move || {
            barrier.wait();
            let _this_task = rendezvous.try_lock().unwrap().take().unwrap();
            barrier.wait();
            dbg!(s);
            barrier.wait();
        });
        *rendezvous.try_lock().unwrap() = Some(task);
    }
    barrier.wait();
    s.clear();
    s.push_str("Sad");
    barrier.wait();
    barrier.wait();
}
