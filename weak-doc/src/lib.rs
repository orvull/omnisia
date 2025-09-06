//! Weak<T> in Rust — mini-docs + runnable examples
//!
//! TL;DR
//! - `Weak<T>` is a **non-owning** pointer to the allocation behind `Rc<T>` or `Arc<T>`.
//! - It **does not** keep the value alive (doesn’t affect drop timing).
//! - Use `Weak` to **break reference cycles** (e.g., parent↔child graphs).
//! - To access the value, call `.upgrade()` → `Option<Rc<T>>` / `Option<Arc<T>>`.
//! - When the last strong (`Rc`/`Arc`) is dropped, the value is dropped; the allocation
//!   is freed when **both** strong and weak counts reach zero.

use std::{
    cell::RefCell,
    rc::{Rc, Weak as RcWeak},
    sync::{Arc, Weak as ArcWeak, Mutex},
    thread,
    time::Duration,
};

/* ───────────────────────── 1) Basics: Rc::Weak (single-threaded) ───────────────────────── */

pub fn ex_rc_weak_basics() {
    println!("== 1) Rc::Weak basics ==");

    let strong = Rc::new(String::from("hello"));
    let weak: RcWeak<String> = Rc::downgrade(&strong);

    println!("strong_count = {}", Rc::strong_count(&strong)); // 1
    println!("weak_count    = {}", Rc::weak_count(&strong));  // 1 (the `weak` above)

    // Access via upgrade
    if let Some(s) = weak.upgrade() {
        println!("upgrade -> {}", s); // "hello"
        // `s` is an Rc clone; keeps the value alive while in scope
    }

    // Drop the last strong owner
    drop(strong);

    // Now the value is dropped; upgrade fails (allocation may still exist until weak_count==0)
    assert!(weak.upgrade().is_none());
    println!("after drop: upgrade -> None");
}

/* ─────────────── 2) Breaking cycles in graphs: Rc<RefCell<T>> + Rc::Weak ───────────────
   Problem: cycles of Rc cause leaks (values never dropped) because refcount never hits zero.
   Solution: use Weak for back-edges (e.g., child -> parent).
*/

#[derive(Debug)]
struct NodeRc {
    name: String,
    parent: RefCell<RcWeak<NodeRc>>,     // weak back-edge
    children: RefCell<Vec<Rc<NodeRc>>>,  // strong edges to children
}

pub fn ex_rc_cycle_vs_weak() {
    println!("\n== 2) Breaking cycles with Rc::Weak ==");

    // Make a parent and a child
    let parent = Rc::new(NodeRc {
        name: "root".into(),
        parent: RefCell::new(RcWeak::new()),
        children: RefCell::new(vec![]),
    });
    let child = Rc::new(NodeRc {
        name: "leaf".into(),
        parent: RefCell::new(RcWeak::new()),
        children: RefCell::new(vec![]),
    });

    // root -> child (strong)
    parent.children.borrow_mut().push(child.clone());
    // child -> root (WEAK)
    *child.parent.borrow_mut() = Rc::downgrade(&parent);

    println!(
        "counts (parent): strong={}, weak={}",
        Rc::strong_count(&parent),
        Rc::weak_count(&parent)
    );
    println!(
        "counts (child) : strong={}, weak={}",
        Rc::strong_count(&child),
        Rc::weak_count(&child)
    );

    // Drop the strong parent; child only holds a WEAK back-edge, so no cycle leak.
    drop(parent);

    // Child's weak parent pointer cannot keep parent alive:
    if child.parent.borrow().upgrade().is_none() {
        println!("parent has been dropped; weak back-edge is now None");
    }
}

/* ───────────────────── 3) Arc::Weak in multi-threaded graphs ───────────────────── */

#[derive(Debug)]
struct NodeArc {
    name: String,
    parent: Mutex<ArcWeak<NodeArc>>,       // weak back-edge
    children: Mutex<Vec<Arc<NodeArc>>>,    // strong edges
}

pub fn ex_arc_weak_multithread() {
    println!("\n== 3) Arc::Weak in multi-threaded graphs ==");

    let root = Arc::new(NodeArc {
        name: "root".into(),
        parent: Mutex::new(ArcWeak::new()),
        children: Mutex::new(vec![]),
    });
    let leaf = Arc::new(NodeArc {
        name: "leaf".into(),
        parent: Mutex::new(ArcWeak::new()),
        children: Mutex::new(vec![]),
    });

    // root -> leaf (strong)
    root.children.lock().unwrap().push(leaf.clone());
    // leaf -> root (WEAK)
    *leaf.parent.lock().unwrap() = Arc::downgrade(&root);

    println!(
        "counts (root): strong={}, weak={}",
        Arc::strong_count(&root),
        Arc::weak_count(&root)
    );
    println!(
        "counts (leaf): strong={}, weak={}",
        Arc::strong_count(&leaf),
        Arc::weak_count(&leaf)
    );

    // Move leaf to another thread, observe parent after dropping root
    let handle = {
        let leaf = leaf.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            if leaf.parent.lock().unwrap().upgrade().is_none() {
                println!("[thread] root dropped; weak back-edge is None");
            }
        })
    };

    drop(root); // drop strong owner
    handle.join().unwrap();
}

/* ───────────────────── 4) Cache pattern with Weak (auto-expiring) ─────────────────────
   Store Weak pointers in a map so entries auto-expire when values are no longer strongly owned.
*/

use std::collections::HashMap;

#[derive(Debug)]
struct BigThing(&'static str);

pub fn ex_cache_with_weak() {
    println!("\n== 4) Cache with Weak (auto-expiring) ==");
    let mut cache: HashMap<&'static str, RcWeak<BigThing>> = HashMap::new();

    // Insert an object and a weak handle in the cache
    let key = "itemA";
    let strong = Rc::new(BigThing("payload"));
    cache.insert(key, Rc::downgrade(&strong));

    // Later: try to fetch from cache
    match cache.get(key).and_then(|w| w.upgrade()) {
        Some(rc) => println!("cache hit (alive): {:?}", rc),
        None => println!("cache miss / expired"),
    }

    // Drop the only strong owner; cache now holds only a Weak
    drop(strong);

    // Try again: upgrade fails, so evict
    match cache.get(key).and_then(|w| w.upgrade()) {
        Some(_) => {}
        None => {
            println!("cache entry expired; pruning");
            cache.remove(key);
        }
    }
    println!("cache len = {}", cache.len());
}

/* ──────────────────────── 5) Leak demo: strong back-edge (DON'T) ────────────────────────
   This shows how a strong parent<->child reference leaks (cycle). We then fix it with Weak.
*/

#[derive(Debug)]
struct BadNode {
    name: String,
    parent: RefCell<Option<Rc<BadNode>>>,     // ❌ strong back-edge (causes leak)
    children: RefCell<Vec<Rc<BadNode>>>,
}

pub fn ex_leak_then_fix() {
    println!("\n== 5) Cycle leak demo (Rc strong back-edge) vs Weak fix ==");

    // Strong back-edge → leak (we'll observe counts).
    let a = Rc::new(BadNode {
        name: "A".into(),
        parent: RefCell::new(None),
        children: RefCell::new(vec![]),
    });
    let b = Rc::new(BadNode {
        name: "B".into(),
        parent: RefCell::new(None),
        children: RefCell::new(vec![]),
    });

    a.children.borrow_mut().push(b.clone());
    *b.parent.borrow_mut() = Some(a.clone()); // ❌ strong cycle A <-> B

    println!(
        "[bad] A counts: strong={}, weak={}",
        Rc::strong_count(&a),
        Rc::weak_count(&a)
    );
    println!(
        "[bad] B counts: strong={}, weak={}",
        Rc::strong_count(&b),
        Rc::weak_count(&b)
    );

    // Dropping a/b won't drop the inner values because strong counts never hit 0 (cycle).
    // (We won't actually cause a process leak in this demo since it ends here.)

    // ✅ Fix: use Weak for back-edge
    #[derive(Debug)]
    struct GoodNode {
        name: String,
        parent: RefCell<RcWeak<GoodNode>>,   // weak back-edge
        children: RefCell<Vec<Rc<GoodNode>>>,
    }

    let p = Rc::new(GoodNode {
        name: "P".into(),
        parent: RefCell::new(RcWeak::new()),
        children: RefCell::new(vec![]),
    });
    let c = Rc::new(GoodNode {
        name: "C".into(),
        parent: RefCell::new(RcWeak::new()),
        children: RefCell::new(vec![]),
    });
    p.children.borrow_mut().push(c.clone());
    *c.parent.borrow_mut() = Rc::downgrade(&p);

    println!(
        "[good] P counts: strong={}, weak={}",
        Rc::strong_count(&p),
        Rc::weak_count(&p)
    );
    println!(
        "[good] C counts: strong={}, weak={}",
        Rc::strong_count(&c),
        Rc::weak_count(&c)
    );
    // Now dropping P will not be kept alive by C's weak parent reference.
}

/* ───────────────────────────────────────── main ───────────────────────────────────────── */


/* ───────────────────────────── Docs-style notes ─────────────────────────────

WHAT `Weak<T>` IS
- A pointer associated with an `Rc<T>` or `Arc<T>` allocation that **does not own** the value.
- Created with `Rc::downgrade(&rc)` / `Arc::downgrade(&arc)`.
- Upgrade with `.upgrade()` → `Option<Rc<T>>` / `Option<Arc<T>>`.
  - `Some(...)` if at least one **strong** ref exists; `None` if value already dropped.

COUNTS & DROP ORDER
- **strong_count**: number of owning references (`Rc`/`Arc`). When this hits 0 -> **value is dropped**.
- **weak_count**: number of `Weak` pointers (does **not** include the internal, hidden “allocation guard”).
- The **allocation** itself is freed when both counts are 0 (i.e., no strong owners and no weaks left).
- Holding `Weak` does **not** prolong the lifetime of `T`; it only keeps the allocation metadata alive.

WHY USE `Weak`
- Break cycles in graphs (parent↔child). Make back-edges weak so strong counts can reach 0.
- Caches/registries: store `Weak` handles so entries auto-expire when not strongly owned elsewhere.
- Observers: non-owning subscribers that may disappear without coordination (upgrade to check).

SINGLE-THREAD vs MULTI-THREAD
- `Rc<T>`/`Rc::Weak<T>`: single-threaded; not `Send`/`Sync`.
- `Arc<T>`/`Arc::Weak<T>`: thread-safe refcounts; ok to share across threads.
- For mutation with Arc, combine with `Mutex`/`RwLock`/atomics.

PATTERNS / APIS
- Create: `let w = Rc::downgrade(&rc);`
- Upgrade: `if let Some(rc) = w.upgrade() { /* use rc */ }`
- Counts: `Rc::strong_count(&rc)`, `Rc::weak_count(&rc)` (same for Arc).
- Evict dead weaks: filter a list/map of `Weak` by `w.upgrade().is_some()`.

PITFALLS
- Don’t forget to make **exactly the back-edges** weak; two-way strong links leak.
- Be careful not to hold temporary strong clones (from `upgrade()`) longer than necessary if you expect a drop.
- `weak_count` doesn’t include the internal guard; seeing `0` for weak_count doesn’t mean the allocation can be freed if strong_count > 0.

MENTAL MODEL
- Think of `Weak` as a “peekable address book entry” for an `Rc/Arc` allocation:
  - You can look up whether the person still lives there (`upgrade()`).
  - If they’ve moved out (no strong refs), the address card is useless (`None`).
  - The card itself doesn’t keep them living there.

*/ 
