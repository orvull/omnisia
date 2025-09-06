//! Arc<T> mini-docs + runnable examples in one file
//!
//! What is Arc<T>?
//! - Thread-safe reference-counted smart pointer (Atomic Rc).
//! - Enables multiple owners of the same heap value across threads.
//! - Cloning is cheap (increments atomic strong refcount).
//! - Send/Sync depends on T (Arc adds thread-safety for the pointer, not the inner T).
//!
//! Common combos:
//! - Arc<T> alone -> shared immutable ownership across threads
//! - Arc<Mutex<T>> / Arc<RwLock<T>> -> shared + mutable across threads
//! - Arc<Atomic*> -> lock-free shared counters/flags
//! - Arc<Something> + Weak<Something> -> shared graphs/trees without cycles

use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::rc::Rc; // only used in doc contrast
use std::thread;
use std::time::Duration;
use std::sync::Weak;

fn example_basic() {
    println!("== Example 1: Basic Arc usage across threads ==");
    let msg = Arc::new(String::from("hello, world"));

    let mut handles = vec![];
    for i in 0..3 {
        let m = Arc::clone(&msg); // cheap, atomic refcount increment
        handles.push(thread::spawn(move || {
            println!("[thread {i}] {}", m);
        }));
    }
    for h in handles { h.join().unwrap(); }

    println!("strong_count(msg) = {}", Arc::strong_count(&msg));
}

fn example_mutation_with_mutex() {
    println!("\n== Example 2: Shared + mutable with Arc<Mutex<T>> ==");
    let data: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(vec![]));

    let mut handles = vec![];
    for i in 1..=4 {
        let d = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let mut guard = d.lock().unwrap(); // exclusive lock
            guard.push(i);
            // guard dropped here
        }));
    }
    for h in handles { h.join().unwrap(); }

    println!("data = {:?}", *data.lock().unwrap()); // [1,2,3,4] in some order
}

fn example_rwlock_readers_writers() {
    println!("\n== Example 3: Many readers / one writer with Arc<RwLock<T>> ==");
    let num = Arc::new(RwLock::new(0_u64));

    // writer
    {
        let mut w = num.write().unwrap();
        *w += 10;
    }

    // multiple readers in parallel
    let mut handles = vec![];
    for i in 0..3 {
        let n = Arc::clone(&num);
        handles.push(thread::spawn(move || {
            let r = n.read().unwrap();
            println!("[reader {i}] value = {}", *r);
        }));
    }
    for h in handles { h.join().unwrap(); }

    // another write
    {
        let mut w = num.write().unwrap();
        *w += 1;
    }
    println!("final value = {}", *num.read().unwrap());
}

fn example_atomic_counter() {
    println!("\n== Example 4: Lock-free counter with Arc<AtomicUsize> ==");
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..8 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                c.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }
    for h in handles { h.join().unwrap(); }

    println!("count = {}", counter.load(Ordering::Relaxed)); // 8000
}

fn example_try_unwrap() {
    println!("\n== Example 5: Arc::try_unwrap to move out when uniquely owned ==");
    let a = Arc::new(String::from("unique"));
    match Arc::try_unwrap(a) {
        Ok(s) => println!("moved out: {s}"), // refcount == 1 -> success
        Err(_) => println!("still shared"),
    }

    let a = Arc::new(String::from("shared"));
    let a2 = Arc::clone(&a);
    match Arc::try_unwrap(a) {
        Ok(_) => println!("unexpected"),
        Err(arc_back) => {
            println!("cannot unwrap: strong_count = {}", Arc::strong_count(&arc_back));
            drop(a2); // drop the sibling so refcount goes to 1 (not shown further)
        }
    }
}

#[derive(Debug)]
struct GNode {
    name: String,
    children: Mutex<Vec<Arc<GNode>>>, // strong edges to children
    parent: Mutex<Weak<GNode>>,       // weak edge to parent to avoid cycles
}

fn example_weak_to_avoid_cycles() {
    println!("\n== Example 6: Avoid cycles with Arc<Weak<T>> ==");
    let root = Arc::new(GNode {
        name: "root".into(),
        children: Mutex::new(vec![]),
        parent: Mutex::new(Weak::new()),
    });

    let child = Arc::new(GNode {
        name: "child".into(),
        children: Mutex::new(vec![]),
        parent: Mutex::new(Weak::new()),
    });

    // root -> child (strong)
    root.children.lock().unwrap().push(child.clone());
    // child -> root (weak)
    *child.parent.lock().unwrap() = Arc::downgrade(&root);

    println!("root strong_count = {}", Arc::strong_count(&root));   // at least 1
    println!("child strong_count = {}", Arc::strong_count(&child)); // at least 1

    // upgrade weak link while root is alive
    if let Some(parent) = child.parent.lock().unwrap().upgrade() {
        println!("child's parent = {}", parent.name);
    }

    // After dropping root, child's weak parent won't keep it alive:
    drop(root);
    // Give the OS a moment so println! order is nice in some environments
    thread::sleep(Duration::from_millis(10));

    if child.parent.lock().unwrap().upgrade().is_none() {
        println!("child's parent has been dropped (weak link is None)");
    }
}

fn main() {
    example_basic();
    example_mutation_with_mutex();
    example_rwlock_readers_writers();
    example_atomic_counter();
    example_try_unwrap();
    example_weak_to_avoid_cycles();
}

/*
Docs-style notes:

Arc<T> — Thread-safe shared ownership (atomic refcount)
- Arc::new(value)       -> Arc<T>
- Arc::clone(&arc)      -> increments STRONG atomic refcount
- Arc::strong_count(&)  -> number of strong references
- Arc::downgrade(&arc)  -> Weak<T> (weak refs don't keep value alive)
- Weak::upgrade(&weak)  -> Option<Arc<T>> (Some if value still alive)
- Arc::try_unwrap(arc)  -> Result<T, Arc<T>> (move out when unique)

Mutation patterns:
- Arc<T> alone gives shared immutable access.
- For mutation across threads: Arc<Mutex<T>> or Arc<RwLock<T>>.
  - Mutex: one writer at a time (exclusive).
  - RwLock: many readers OR one writer.
- For counters/flags: Arc<Atomic*> (lock-free).

Threading:
- Arc<T> is Send + Sync if T: Send + Sync (Arc doesn't auto-make T thread-safe).
- Use Mutex/RwLock/Atomics to protect interior mutation in multi-threaded code.

Contrast:
- Rc<T>: single-threaded refcount (non-atomic), !Send, !Sync.
- Arc<T>: multi-threaded refcount (atomic), Send/Sync if T is.
- Box<T>: single owner, no refcount; immediate drop on owner drop.

Pitfalls:
- Avoid holding locks longer than needed to prevent contention/deadlocks.
- Be careful with RwLock writer starvation (implementation-dependent).
- Weak<T> is essential to break cycles in graph-like structures.

*/
