use std::sync::{Arc, Mutex};

use leak_playground_std::sync::mpsc::rendezvous_channel;
use tokio::sync::Barrier;

// #[tokio::main]
// async fn main() {
//     let mut s = "Hello world".to_owned();
//     let barrier = Arc::new(Barrier::new(2));
//     {
//         let (tx, rx) = rendezvous_channel();
//         let barrier = Arc::clone(&barrier);
//         let s = s.as_str();

//         let task = leak_playground_tokio::task::spawn_scoped(async move {
//             let _this_task = rx.recv().unwrap();
//             barrier.wait().await;
//             dbg!(s);
//             barrier.wait().await;
//         });
//         tx.send(task).unwrap();
//     }
//     s.clear();
//     s.push_str("Sad");
//     barrier.wait().await;
//     barrier.wait().await;
// }

#[tokio::main]
async fn main() {
    let mut s = "Hello world".to_owned();
    let barrier = Arc::new(Barrier::new(2));
    {
        let rendezvous = Mutex::new(None);
        let rendezvous = &rendezvous;
        let barrier = Arc::clone(&barrier);
        let s = s.as_str();

        let task = leak_playground_tokio::task::spawn_scoped(async move {
            barrier.wait().await;
            let _this_task = rendezvous.try_lock().unwrap().take().unwrap();
            barrier.wait().await;
            dbg!(s);
            barrier.wait().await;
        });
        *rendezvous.try_lock().unwrap() = Some(task);
    }
    barrier.wait().await;
    s.clear();
    s.push_str("Sad");
    barrier.wait().await;
    barrier.wait().await;
}
