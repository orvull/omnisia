use async_doc::{
    ex_basic,
    ex_spawn_and_join,
    ex_joinset_and_cancel,
    ex_channels,
    ex_locks_notify_semaphore,
    ex_timeouts_and_select,
    ex_streams,
    ex_blocking_work,
};

#[tokio::main]
async fn main() {
    ex_basic().await;
    ex_spawn_and_join().await;
    ex_joinset_and_cancel().await;
    ex_channels().await;
    ex_locks_notify_semaphore().await;
    ex_timeouts_and_select().await;
    ex_streams().await;
    ex_blocking_work().await;
}
