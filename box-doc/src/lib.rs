//! Simple docs + examples for Box<T>

use std::fmt::Debug;

//
// Example 1: Owning a simple value
//
pub fn example_basic() {
    let b = Box::new(42); // allocate i32 on the heap

    println!("value = {}", b);      // auto-deref
    println!("deref  = {}", *b);    // manual deref, moves if not Copy
}

//
// Example 2: Recursive data type
//
enum List {
    Node(i32, Box<List>), // recursive type only possible with Box
    Nil,
}

pub fn example_recursive() {
    let list = List::Node(1, Box::new(List::Node(2, Box::new(List::Nil))));
    match list {
        List::Node(v, _) => println!("first node = {}", v),
        List::Nil => println!("empty"),
    }
}

//
// Example 3: Trait objects
//
trait Animal {
    fn speak(&self);
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) { println!("Woof!"); }
}

struct Cat;
impl Animal for Cat {
    fn speak(&self) { println!("Meow!"); }
}

pub fn example_trait_objects() {
    let animals: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cat)];
    for a in animals {
        a.speak(); // dynamic dispatch
    }
}

//
// Example 4: Borrow without moving
//
pub fn example_borrow() {
    let b = Box::new(String::from("hello"));

    let r: &String = b.as_ref(); // borrow immutably, don’t move out
    println!("borrow = {}", r);

    // still can use b afterwards
    println!("again   = {}", b);
}

//
// Docs-style comparison (for humans)
//
/*
| `Box<T>`                           | Use case                                      |
| ---------------------------------- | --------------------------------------------- |
| Single ownership                   | Exactly one owner of the heap value           |
| Heap allocation                    | Moves large/recursive data off stack          |
| Auto cleanup                       | Freed when box is dropped                     |
| Move on deref (`*b`)               | Moves (unless `Copy`), consumes box           |
| Borrow (`&*b`, `as_ref`, `as_mut`) | Safe way to inspect/modify without moving     |
| Thread safety                      | Same as `T` (box doesn’t add sync/atomic)     |
*/

//
// Internal view (simplified):
//
// pub struct Box<T: ?Sized> {
//     ptr: Unique<T>, // raw heap pointer with ownership
// }
//
// Drop impl for Box<T> calls drop on value, then deallocates heap memory.
//

