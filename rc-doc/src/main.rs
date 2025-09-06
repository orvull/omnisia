//! Rc<T> mini-docs + runnable examples in one file
//!
//! What is Rc<T>?
//! - Single-threaded reference-counted smart pointer.
//! - Enables multiple owners of the same heap value.
//! - Cloning is cheap (increments strong refcount).
//! - Not thread-safe (use Arc<T> for multi-threading).
//!
//! Common combos:
//! - Rc<T> alone -> shared immutable ownership
//! - Rc<RefCell<T>> -> shared + interior-mutable (single-thread)
//! - Rc<Something> + Weak<Something> -> shared graphs without cycles

use std::cell::RefCell;
use std::rc::{Rc, Weak};

fn example_basic() {
    println!("== Example 1: Basic Rc usage ==");
    let a = Rc::new("hello".to_string());

    let b = Rc::clone(&a); // same as a.clone()
    let c = a.clone();

    println!("a = {a}, b = {b}, c = {c}");
    println!("strong_count(a) = {}", Rc::strong_count(&a)); // 3

    drop(b);
    println!("after drop(b), strong_count(a) = {}", Rc::strong_count(&a)); // 2

    drop(c);
    println!("after drop(c), strong_count(a) = {}", Rc::strong_count(&a)); // 1
}

#[derive(Debug)]
struct Node {
    value: i32,
    next: Option<Rc<Node>>, // many parents can point to the same child
}

fn example_tree_like_sharing() {
    println!("\n== Example 2: Tree-like sharing with Rc ==");
    let leaf = Rc::new(Node { value: 1, next: None });

    let branch1 = Rc::new(Node { value: 2, next: Some(leaf.clone()) });
    let branch2 = Rc::new(Node { value: 3, next: Some(leaf.clone()) });

    println!("branch1 -> {:?}", branch1.next.as_ref().unwrap());
    println!("branch2 -> {:?}", branch2.next.as_ref().unwrap());
    println!("leaf strong_count = {}", Rc::strong_count(&leaf)); // 3 (leaf, branch1, branch2)
}

fn example_mutation_with_refcell() {
    println!("\n== Example 3: Shared + mutable with Rc<RefCell<T>> ==");
    let numbers: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(vec![1, 2, 3]));

    let a = numbers.clone();
    let b = numbers.clone();

    // mutate via one owner
    a.borrow_mut().push(4);

    // observe via another owner
    println!("b sees {:?}", b.borrow()); // [1, 2, 3, 4]
    println!("strong_count(numbers) = {}", Rc::strong_count(&numbers)); // 3
}

#[derive(Debug)]
struct GraphNode {
    name: String,
    // Strong edges to children:
    children: RefCell<Vec<Rc<GraphNode>>>,
    // Weak edge to parent to avoid reference cycle:
    parent: RefCell<Weak<GraphNode>>,
}

fn example_weak_to_avoid_cycles() {
    println!("\n== Example 4: Avoid cycles with Weak ==");
    let root = Rc::new(GraphNode {
        name: "root".into(),
        children: RefCell::new(Vec::new()),
        parent: RefCell::new(Weak::new()),
    });

    let child = Rc::new(GraphNode {
        name: "child".into(),
        children: RefCell::new(Vec::new()),
        parent: RefCell::new(Weak::new()),
    });

    // root --(strong)--> child
    root.children.borrow_mut().push(child.clone());

    // child --(weak)--> root
    *child.parent.borrow_mut() = Rc::downgrade(&root);

    println!("root strong_count = {}", Rc::strong_count(&root));   // at least 1 (root) + maybe others
    println!("child strong_count = {}", Rc::strong_count(&child)); // at least 1 (child)

    // Upgrade weak parent pointer (if alive)
    if let Some(parent_rc) = child.parent.borrow().upgrade() {
        println!("child's parent = {}", parent_rc.name);
    } else {
        println!("child's parent already dropped");
    }

    // Dropping root would not leak because child's parent is only a Weak reference
    // (no strong cycle). After drop(root), child.parent.upgrade() would be None.
}

fn main() {
    example_basic();
    example_tree_like_sharing();
    example_mutation_with_refcell();
    example_weak_to_avoid_cycles();
}

/*
Docs-style notes:

Rc<T> — Single-threaded shared ownership
- Rc::new(value) -> Rc<T>
- Rc::clone(&rc) or rc.clone() -> increments strong refcount
- Rc::strong_count(&rc) -> current strong refs
- Rc::downgrade(&rc) -> Weak<T> (weak ref does NOT keep value alive)
- Weak::upgrade(&weak) -> Option<Rc<T>> (Some if value still alive)

Rc vs Box:
- Box<T>: single owner, no refcount overhead, drops immediately when owner drops
- Rc<T>: multiple owners, small overhead for refcount, drops when count hits zero

Mutation pattern:
- Rc<T> only gives shared immutable access by default
- For mutation, wrap inner value: Rc<RefCell<T>>
  - .borrow_mut() / .borrow() have runtime-checked borrowing (can panic if overlapped)
  - Keep borrows short; prefer try_borrow* if you want to avoid panics

Avoiding cycles:
- Graphs/trees with parent <-> child links can create Rc cycles -> memory leak
- Use Weak<T> for back-edges (parents) to break cycles

Threading:
- Rc<T> is !Send and !Sync (not thread-safe)
- For multi-threaded shared ownership, use Arc<T> instead
*/
