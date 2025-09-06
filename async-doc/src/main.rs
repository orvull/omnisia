//! Async in Rust with Tokio — mini-docs + runnable examples
//!
//! TL;DR
//! - `async fn` returns a Future; `.await` drives it to completion on a runtime.
//! - You need an executor (e.g., Tokio) to poll futures.
//! - Don’t block the async thread: use `tokio::time`, `tokio::sync`, `spawn_blocking`.
//! - Prefer `tokio::sync::Mutex/RwLock` in async code (std::sync::Mutex blocks the thread).
//!
//! This file demonstrates:
//!  1) async/await basics
//!  2) spawning tasks, join handles, JoinSet, cancellation
//!  3) channels (mpsc / oneshot), async Mutex/RwLock/Notify/Semaphore
//!  4) timeouts, `select!`, cancellation points
//!  5) streams
//!  6) blocking work offloaded safely
//!  7) brief internals & API cheat sheet (at bottom)

use futures::{stream, StreamExt};
use tokio::{
    sync::{mpsc, oneshot, Mutex, RwLock, Notify, Semaphore},
    task::{JoinSet},
    time::{self, Duration},
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

/* ─────────────────────────── 1) Basics ─────────────────────────── */

async fn ex_basic() {
    println!("== 1) async/await basics ==");
    async fn add_after(a: i32, b: i32, ms: u64) -> i32 {
        time::sleep(Duration::from_millis(ms)).await;
        a + b
    }

    let x = add_after(1, 2, 20).await; // await suspends here, yields to runtime
    println!("1+2 after delay = {x}");
}

/* ───────────────────── 2) Spawn tasks & join ───────────────────── */

async fn ex_spawn_and_join() {
    println!("\n== 2) spawning tasks & joining ==");
    // Tasks run concurrently on Tokio worker threads
    let h1 = tokio::spawn(async {
        time::sleep(Duration::from_millis(10)).await;
        "task-1 done"
    });
    let h2 = tokio::spawn(async {
        time::sleep(Duration::from_millis(5)).await;
        "task-2 done"
    });

    // Await both (order independent)
    let (r1, r2) = tokio::join!(h1, h2);
    println!("{}, {}", r1.unwrap(), r2.unwrap());
}

/* ─────────────── 3) JoinSet and cancellation patterns ───────────── */

async fn ex_joinset_and_cancel() {
    println!("\n== 3) JoinSet & cancellation ==");
    let mut set = JoinSet::new();
    for i in 0..3 {
        set.spawn(async move {
            time::sleep(Duration::from_millis(10 * (i + 1))).await;
            format!("job {i} done")
        });
    }

    // Cancel one task (e.g., based on condition)
    if let Some(handle) = set.join_next().await {
        println!("first finished: {}", handle.unwrap());
        // drop remaining by clearing the set:
        set.abort_all();
    }

    // Draining aborted joins (optional)
    while let Some(res) = set.join_next().await {
        match res {
            Ok(msg) => println!("finished later: {msg}"),
            Err(e) if e.is_cancelled() => println!("a task was cancelled"),
            Err(e) => println!("task error: {e}"),
        }
    }
}

/* ───────────── 4) Channels (mpsc / oneshot) ───────────── */

async fn ex_channels() {
    println!("\n== 4) channels (mpsc & oneshot) ==");

    // mpsc: multiple producers, single consumer
    let (tx, mut rx) = mpsc::channel::<String>(16);
    for i in 0..3 {
        let txi = tx.clone();
        tokio::spawn(async move {
            txi.send(format!("msg #{i}")).await.ok();
        });
    }
    drop(tx); // close channel—receiver ends when all senders dropped

    while let Some(msg) = rx.recv().await {
        println!("mpsc got: {msg}");
    }

    // oneshot: one value, one receiver
    let (txo, rxo) = oneshot::channel::<u32>();
    tokio::spawn(async move {
        time::sleep(Duration::from_millis(10)).await;
        let _ = txo.send(42);
    });
    println!("oneshot => {}", rxo.await.unwrap());
}

/* ──────── 5) Async locks (Mutex/RwLock), Notify, Semaphore ──────── */

async fn ex_locks_notify_semaphore() {
    println!("\n== 5) async locks, notify, semaphore ==");
    // Async Mutex (non-blocking while pending)
    let counter = Mutex::new(0u64);
    let mut tasks = vec![];
    for _ in 0..4 {
        let c = &counter;
        tasks.push(tokio::spawn(async move {
            for _ in 0..1000 {
                *c.lock().await += 1;
            }
        }));
    }
    for t in tasks { t.await.unwrap(); }
    println!("counter (Mutex) = {}", *counter.lock().await);

    // Async RwLock
    let data = RwLock::new(String::from("init"));
    {
        let mut w = data.write().await;
        *w = "updated".into();
    }
    let (r1, r2, r3) = tokio::join!(
        async { data.read().await.clone() },
        async { data.read().await.clone() },
        async { data.read().await.clone() },
    );
    println!("RwLock reads: {r1:?}, {r2:?}, {r3:?}");

    // Notify: simple wakeup primitive
    let notify = Notify::new();
    let notified = notify.notified();
    let n2 = notify.clone();
    tokio::spawn(async move {
        time::sleep(Duration::from_millis(15)).await;
        n2.notify_one();
    });
    notified.await; // wait for notification
    println!("notified!");

    // Semaphore: rate limiting / resource permits
    let sem = Semaphore::new(2); // two concurrent permits
    let mut handles = vec![];
    for i in 0..5 {
        let s = sem.clone();
        handles.push(tokio::spawn(async move {
            let _permit = s.acquire().await.unwrap();
            println!("task {i} entered");
            time::sleep(Duration::from_millis(10)).await;
            println!("task {i} leaving");
        }));
    }
    for h in handles { h.await.unwrap(); }
}

/* ─────────────── 6) Timeouts, select!, cancellation ─────────────── */

async fn ex_timeouts_and_select() {
    use tokio::select;
    println!("\n== 6) timeout & select! ==");

    // timeout
    let res = tokio::time::timeout(Duration::from_millis(10), work_slow(20)).await;
    println!("timeout result: {:?}", res.map_err(|_| "timed out"));

    // select! over multiple branches
    let fast = work_slow(5);
    let slow = work_slow(30);
    select! {
        v = fast => println!("fast finished first: {v}"),
        v = slow => println!("slow finished first: {v}"),
        _ = time::sleep(Duration::from_millis(1_000)) => println!("fallback timeout"),
    }

    // Cancellation: dropping a future cancels it
    let handle = tokio::spawn(work_slow(1000)); // long task
    time::sleep(Duration::from_millis(5)).await;
    handle.abort(); // cooperative cancel
    match handle.await {
        Err(e) if e.is_cancelled() => println!("task cancelled"),
        other => println!("task finished: {:?}", other),
    }
}

async fn work_slow(ms: u64) -> &'static str {
    time::sleep(Duration::from_millis(ms)).await;
    "ok"
}

/* ───────────────────────── 7) Streams ───────────────────────── */

async fn ex_streams() {
    println!("\n== 7) streams (async sequences) ==");
    // A tiny stream of numbers with async processing
    let s = stream::iter(1..=5).then(|n| async move {
        time::sleep(Duration::from_millis(5)).await;
        n * n
    });

    let out: Vec<_> = s.collect().await;
    println!("squares via stream = {:?}", out);
}

/* ─────────────── 8) Offloading blocking work safely ─────────────── */

async fn ex_blocking_work() {
    println!("\n== 8) spawn_blocking for CPU-bound or blocking IO ==");
    // Heavy/blocking computations must not run on async worker threads.
    let sum = tokio::task::spawn_blocking(|| {
        // pretend heavy CPU loop
        (0..2_000_000).fold(0u64, |a, b| a.wrapping_add(b))
    }).await.unwrap();
    println!("blocking sum = {sum}");
}

/* ────────────────────────── Docs-style notes ──────────────────────────

WHAT ASYNC IS
- `async fn foo() -> T` returns `impl Future<Output = T>`.
- `.await` yields control to the runtime until the Future is ready.
- Futures are polled cooperatively; no preemption. Long computations must yield (e.g., via `.await`).

RUNTIME
- The standard library provides Futures but NOT an executor. Use a runtime (Tokio, async-std, etc.).
- Tokio features: multithreaded scheduler, timers, IO, sync primitives, task utilities.

DO & DON’T
- ✅ Use `tokio::time::sleep`, channels, async `Mutex/RwLock`, `Notify`, `Semaphore`.
- ✅ Use `spawn_blocking` for CPU-bound or blocking syscalls (file compression, synchronous DB client).
- ❌ Don’t hold a std::sync::Mutex or a long-lived borrow across `.await` (can deadlock/starve).
- ❌ Don’t block with `std::thread::sleep` in async code.

LOCKS IN ASYNC
- Use `tokio::sync::Mutex/RwLock`: `lock().await` returns a guard; keep lock scope small; drop before `.await`ing other things.
- For simple counters/flags use atomics on a shared `Arc<Atomic*>` (works fine inside async).

CONCURRENCY PRIMITIVES (Tokio)
- Tasks: `tokio::spawn`, `JoinSet`, `JoinHandle::abort`.
- Time: `tokio::time::{sleep, timeout, interval}`.
- Select: `tokio::select!` to await whichever future completes first.
- Channels: `mpsc` (multi-producer), `oneshot` (single value).
- Sync: `Mutex`, `RwLock`, `Notify` (wakeup), `Semaphore` (permits).

CANCELLATION
- Dropping a future or calling `JoinHandle::abort()` cancels the task.
- Cancellation is cooperative: code should `.await` periodically to be cancellable.

STREAMS
- A stream is “async Iterator”. Common ops via `futures::stream`: `map/then/buffer_unordered/collect`.
- Many IO types in Tokio implement Stream (e.g., lines from a socket via `Framed` in tokio-util).

INTEROP & TRAITS
- Trait methods can’t be `async` in stable without help; use `async-trait` crate or GATs-based patterns.
- `Send` across `.await`: values held over an `.await` inside a `Send` future must be `Send`.

API CHEAT SHEET
- Spawn:          `tokio::spawn(async move { ... }) -> JoinHandle<T>`
- JoinSet:        run many tasks, `set.spawn(...)`, `set.join_next().await`, `set.abort_all()`
- Timeout:        `tokio::time::timeout(dur, fut).await`
- Select:         `tokio::select! { a = fut1 => ..., _ = fut2 => ..., }`
- Channels:       `let (tx, rx) = mpsc::channel::<T>(cap); tx.send(v).await; rx.recv().await;`
- Oneshoot:       `let (tx, rx) = oneshot::channel(); tx.send(v)?; rx.await?;`
- Locks:          `let mut g = mutex.lock().await;`
- Notify:         `notify.notified().await; notify.notify_one();`
- Semaphore:      `let permit = sem.acquire().await?;` (drops to release)
- Blocking:       `tokio::task::spawn_blocking(|| heavy())`

INTERNALS (mental model)
- `async fn` is transformed to a state machine that implements `Future`.
- Each `.await` is a suspension point; the executor polls the future when it can make progress.
- Tokio uses a work-stealing scheduler; IO/timers wake tasks via reactor events.

*/```
