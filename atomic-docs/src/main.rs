//! Atomics in Rust — std::sync::atomic::Atomic* + crossbeam::atomic::AtomicCell<T>
//!
//! TL;DR
//! - Atomics let threads coordinate **without locks** by reading/writing plain machine words atomically.
//! - The standard library offers fixed atomic types: AtomicBool, AtomicUsize, AtomicI64, …, AtomicPtr<T>.
//! - `AtomicCell<T>` (from `crossbeam`) is a convenient wrapper that works for any `T: Copy` (and some Option<NonNull<_>>).
//! - Memory order matters: use Relaxed for counters, Acquire/Release for handoffs, SeqCst for simplicity (slower).
//!
//! This file demonstrates:
//!  1) Atomic counters with Relaxed
//!  2) Flags with Acquire/Release
//!  3) `compare_exchange` patterns (one-time init / CAS loop)
//!  4) AtomicPtr and fences
//!  5) AtomicCell<T> ergonomics (load/store/swap/update)
//!  6) Cheatsheet + pitfalls (in comments)

use std::{
    ptr::NonNull,
    sync::{
        atomic::{
            fence, AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering::{self, *}
        },
        Arc,
    },
    thread,
    time::Duration,
};

// Crossbeam's AtomicCell:
use crossbeam::atomic::AtomicCell;

/* ───────────────────────── 1) Counter (Relaxed) ─────────────────────────
Relaxed operations are fine when you only need a number to be correct,
not to publish data associated with that number.
*/
fn ex_relaxed_counter() {
    println!("== 1) Relaxed counter ==");
    let hits = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    for _ in 0..8 {
        let h = hits.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100_000 {
                h.fetch_add(1, Relaxed);
            }
        }));
    }
    for h in handles { h.join().unwrap(); }
    println!("hits = {}", hits.load(Relaxed)); // 800_000
}

/* ───────────── 2) Flag handoff (Release writer → Acquire reader) ─────────────
Use a Release store to "publish" data written before it; Acquire load by reader
to "see" that data. This creates a happens-before edge.
*/
fn ex_acquire_release_flag() {
    println!("\n== 2) Acquire/Release flag (publish data) ==");
    static READY: AtomicBool = AtomicBool::new(false);
    static VALUE: AtomicU64  = AtomicU64::new(0);

    let w = thread::spawn(|| {
        VALUE.store(42, Relaxed);      // write data first
        READY.store(true, Release);    // publish with Release
    });

    let r = thread::spawn(|| {
        // Spin until we observe the flag with Acquire
        while !READY.load(Acquire) { std::hint::spin_loop(); }
        // Because of Acquire, we must observe VALUE=42 here
        let v = VALUE.load(Relaxed);
        println!("observed VALUE = {}", v);
    });

    w.join().unwrap();
    r.join().unwrap();
}

/* ─────────────── 3) CAS patterns: one-time init & CAS loop ───────────────
- compare_exchange(old, new, success_order, failure_order) attempts to set atom from old→new.
- On success returns Ok(old); on failure returns Err(current).
*/
fn ex_compare_exchange() {
    println!("\n== 3) compare_exchange patterns ==");
    // One-time init (idempotent set from 0 → some id)
    let id = AtomicUsize::new(0);
    let my_id = 7;
    let _ = id.compare_exchange(0, my_id, AcqRel, Acquire).ok();
    println!("id after one-time init = {}", id.load(Acquire));

    // CAS loop: increment even-only (toy example)
    let x = AtomicUsize::new(10);
    loop {
        let cur = x.load(Relaxed);
        if cur % 2 == 1 {
            println!("x is odd; not changing");
            break;
        }
        // propose next even+2
        match x.compare_exchange_weak(cur, cur + 2, AcqRel, Acquire) {
            Ok(_) => { println!("x -> {}", x.load(Relaxed)); break; }
            Err(_) => { std::hint::spin_loop(); } // retry
        }
    }
}

/* ─────────────── 4) AtomicPtr + fences (advanced publish) ───────────────
Sometimes you publish *pointers*. Use Release on the publishing store and
Acquire on the consuming load. `fence(Release)` / `fence(Acquire)` can be
used to separate the atomic op from adjacent ordinary memory accesses.
*/
fn ex_atomic_ptr_and_fence() {
    println!("\n== 4) AtomicPtr & fences ==");
    #[derive(Debug)]
    struct Payload { a: u32, b: u32 }

    static PTR: AtomicPtr<Payload> = AtomicPtr::new(std::ptr::null_mut());

    // Producer thread: allocate and publish
    let t = thread::spawn(|| {
        let b = Box::new(Payload { a: 1, b: 2 });
        let raw = Box::into_raw(b);
        // Ensure prior writes to *raw are visible before we publish the pointer:
        fence(Release);
        PTR.store(raw, Release);
    });

    // Consumer: wait until pointer is non-null, then read it
    let r = thread::spawn(|| {
        let mut p;
        loop {
            p = PTR.load(Acquire);
            if !p.is_null() { break; }
            std::hint::spin_loop();
        }
        // Acquire (and the Release fence) ensure we see initialized fields.
        let val = unsafe { &*p };
        println!("read via ptr: {:?}", val);
        // Clean-up: reclaim the Box (single consumer in this toy demo)
        unsafe { drop(Box::from_raw(p)); }
        PTR.store(std::ptr::null_mut(), Release);
    });

    t.join().unwrap();
    r.join().unwrap();
}

/* ─────────────────── 5) AtomicCell<T> ergonomics (crossbeam) ───────────────────
AtomicCell<T> provides load/store/swap/fetch_update for any `T: Copy` (and some niche
non-Copy via specialized impls). It’s often simpler than juggling specific Atomic* types.
*/
fn ex_atomic_cell_basics() {
    println!("\n== 5) AtomicCell<T> basics ==");
    let cell = AtomicCell::new(10u32);
    println!("load = {}", cell.load());
    cell.store(20);
    println!("after store = {}", cell.load());
    let old = cell.swap(30);
    println!("swap: old={}, new={}", old, cell.load());

    // fetch_update: CAS with a closure (retry loop inside)
    let res = cell.fetch_update(Relaxed, Relaxed, |cur| {
        if cur < 100 { Some(cur + 1) } else { None }
    });
    println!("fetch_update -> {:?}, now={}", res, cell.load());

    // Works for other Copy types, including pointers:
    let mut v = 5i32;
    let p = NonNull::new(&mut v as *mut i32).unwrap();
    let pcell = AtomicCell::new(Some(p));
    let got = pcell.load();
    if let Some(nn) = got { unsafe { *nn.as_ptr() += 1; } }
    println!("v after AtomicCell<NonNull> = {}", v);
}

/* ─────────────── 5b) Arc<AtomicCell<T>> across threads ───────────────
AtomicCell methods take &self and do atomic ops internally, so sharing with Arc is easy.
*/
fn ex_atomic_cell_threads() {
    println!("\n== 5b) AtomicCell<T> across threads ==");
    let sum = Arc::new(AtomicCell::new(0u64));
    let mut handles = vec![];
    for _ in 0..4 {
        let s = sum.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..250_000 {
                // emulate fetch_add via update loop
                s.fetch_add(1); // (AtomicCell has fetch_add for numeric T)
            }
        }));
    }
    for h in handles { h.join().unwrap(); }
    println!("sum = {}", sum.load());
}

/* Convenience: provide a small extension when the crate version has numeric ops.
   Recent crossbeam exposes fetch_add/fetch_sub for numeric T; if your version
   lacks it, you can emulate with fetch_update. */
trait FetchAdd {
    fn fetch_add(&self, x: u64) -> u64;
}
impl FetchAdd for AtomicCell<u64> {
    fn fetch_add(&self, x: u64) -> u64 {
        self.fetch_update(Relaxed, Relaxed, |cur| Some(cur.wrapping_add(x)))
            .unwrap_or_else(|cur| cur)
    }
}

fn main() {
    ex_relaxed_counter();
    ex_acquire_release_flag();
    ex_compare_exchange();
    ex_atomic_ptr_and_fence();
    ex_atomic_cell_basics();
    ex_atomic_cell_threads();

    println!("\n== Cheatsheet (see comments below) ==");
}

/* ───────────────────────────── Docs-style notes ─────────────────────────────

STANDARD ATOMICS
- Types: AtomicBool, AtomicI*/U*, AtomicPtr<T>, etc. Size matches the underlying type.
- Basic ops: load(Ordering), store(val, Ordering), swap(val, Ordering),
             fetch_add/sub/and/or/xor, compare_exchange / compare_exchange_weak.

ORDERINGS (from weakest to strongest)
- Relaxed: atomicity only. No cross-thread visibility guarantees beyond the op itself.
  Use for counters/IDs where you don’t read other data guarded by the atomic.
- Acquire (loads): after an Acquire load **succeeds**, subsequent reads in this thread
  see writes that were **before** a matching Release store in the other thread.
- Release (stores): publishes prior writes before the store becomes visible.
- AcqRel: both sides of a RMW op (e.g., CAS).
- SeqCst: total global ordering on all SeqCst ops. Easiest but may be slower.

RULES OF THUMB
- **Counter only** → Relaxed.
- **Flag + data handoff** → writer uses Release store; reader uses Acquire load.
- **CAS loop** → success: AcqRel, failure: Acquire (common pattern).
- **Unsure** → start with SeqCst for correctness, then relax if really needed.

FENCES
- `fence(Ordering)` adds a memory barrier **without** touching an atomic location.
  Rarely needed; use when you must separate ordinary memory accesses from the atomic op.

ATOMICPTR
- Use Release store to publish a fully-initialized object; readers use Acquire load.
- Manage ownership carefully (who frees the allocation?).

ATOMICCELL<T> (crossbeam)
- Works for any `T: Copy` (+ a few special cases). API: new, load, store, swap,
  fetch_update, take, into_inner, etc. Some versions include numeric fetch_add/sub.
- Internally chooses the best representation (locks if needed for wider types/targets).
- Easier ergonomics than raw `Atomic*` for small Copy payloads (u64, bool, NonNull, etc).
- Still obeys memory orderings — the methods accept `Ordering` or specify relaxed defaults.

PITFALLS
- **Relaxed is not a publish**: if you write data then set a flag with Relaxed, another
  thread might see the flag but not the data.
- **Holding references**: Don’t read a pointer atomically and then use it after another
  thread might have freed it. Pair atomics with ownership protocols (hazard pointers,
  epochs, RCU) or make sure only one party frees.
- **ABA problem**: CAS can be fooled if a value changes A→B→A. Use tagged pointers or
  sequence counters when necessary.
- **Spin without backoff**: use `std::hint::spin_loop()` in tight CAS loops, or prefer channels/locks when appropriate.

WHEN TO USE ATOMICS VS LOCKS
- Atomics: simple flags/counters, low-contention single-word state, high-performance data structures by experts.
- Locks: complex invariants or multi-field state; safer and often fast enough.

CHEAT SHEET
- Counter (fast):            `fetch_add(1, Relaxed)`
- Publish data:              `data.store(..., Relaxed); flag.store(true, Release)`
- Observe published data:    `while !flag.load(Acquire) {}`; then read `data`
- One-time init (CAS):       `cas(0, new, AcqRel, Acquire)`
- AtomicCell number bump:    `cell.fetch_update(Relaxed, Relaxed, |x| Some(x+1))`
- Pointer publish:           `fence(Release); AP.store(ptr, Release)`

*/ 
