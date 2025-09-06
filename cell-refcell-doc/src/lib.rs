use std::cell::{RefCell, Cell};

struct CellCounter {
    count: Cell<u32>, // interior mutability
}

impl CellCounter {
    fn tick(&self) {
        self.count.set(self.count.get() + 1); // mutate through &self
    }

    fn get(&self) -> u32 {
        self.count.get()
    }
}


struct RefCellCounter {
    history: RefCell<Vec<u32>>, // interior mutability for a collection
}

impl RefCellCounter {
    fn tick(&self) {
        let mut vec = self.history.borrow_mut(); // runtime-checked mutable borrow
        let new_val = vec.last().unwrap_or(&0) + 1;
        vec.push(new_val);
    }

    fn last(&self) -> u32 {
        let vec = self.history.borrow(); // runtime-checked immutable borrow
        *vec.last().unwrap_or(&0)
    }

    fn all(&self) -> Vec<u32> {
        self.history.borrow().clone() // clone so we can return
    }
}
pub fn cell_example() {
    let c = CellCounter { count: Cell::new(0) };
    c.tick();
    c.tick();
    println!("count = {}", c.get());
}

pub fn refcell_example() {
    let c = RefCellCounter {
        history: RefCell::new(vec![]),
    };
    c.tick();
    c.tick();
    c.tick();
    println!("Last = {}", c.last());
    println!("All  = {:?}", c.all());
}

/* 

| `Cell<T>`                          | `RefCell<T>`                               |
| ---------------------------------- | ------------------------------------------ |
| Only for `Copy` or moveable values | Works for any type (like `Vec`, `HashMap`) |
| Get/Set value only (no refs)       | Borrow refs (`&T` / `&mut T`) at runtime   |
| Zero overhead, very fast           | Slight runtime cost, may panic if misused  |


pub struct Cell<T> {
    value: UnsafeCell<T>, // wrapper that disables borrow checker
}
UnsafeCell<T> is the only legal way in Rust to do “interior mutability” at the compiler level.

Cell::get/set just copies values in and out directly (requires Copy unless you move).

pub struct RefCell<T> {
    borrow: Cell<BorrowFlag>, // small counter/flag
    value: UnsafeCell<T>,     // the wrapped value
}

type BorrowFlag = isize; // in stdlib it’s usually an isize
// 0     => not borrowed
// >0    => number of active immutable borrows
// -1    => mutably borrowed

borrow() increments the counter.

borrow_mut() checks counter == 0, then sets it to -1.

drop of Ref/RefMut decrements/reset the counter.

If rules violated → panic.

So:

Cell<T> = just UnsafeCell<T>, no borrow tracking.

RefCell<T> = UnsafeCell<T> + a borrow counter.

Their runtime borrow-checking is not atomic → two threads could borrow at the same time, breaking safety.
*/