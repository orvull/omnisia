//! Iterators in Rust: docs + runnable examples
//!
//! What is an Iterator?
//! - An object that produces a sequence of values on demand.
//! - In Rust: anything implementing the `Iterator` trait.
//!
//! Core API (from std::iter):
//! trait Iterator {
//!     type Item;
//!     fn next(&mut self) -> Option<Self::Item>;
//!     // default methods (map, filter, fold, etc.) are built on top of next()
//! }
//!
//! - Lazy: iterator adapters (map, filter, etc.) build new iterators.
//! - Consuming adapters (collect, for_each, sum, etc.) pull values and end iteration.

fn example_basic() {
    println!("== Example 1: Iteration entry points ==");

    let v = vec![10, 20, 30];

    // 1. into_iter() -> takes ownership of the collection
    //    - The vector itself is consumed.
    //    - Yields owned values (T).
    //    - After calling into_iter(), you cannot use v again unless it’s cloned before.
    for x in v.clone().into_iter() {
        println!("into_iter got {}", x);
    }
    // println!("{:?}", v); // ❌ cannot use here, v moved

    // 2. iter() -> borrows immutably
    //    - The vector is not consumed.
    //    - Yields &T (references to elements).
    //    - You can still use v afterwards, since it’s only borrowed.
    for x in v.iter() {
        println!("iter got {}", x);
    }
    println!("v is still usable after iter(): {:?}", v);

    // 3. iter_mut() -> borrows mutably
    //    - The vector is not consumed.
    //    - Yields &mut T (mutable references).
    //    - You can modify elements through the iterator.
    let mut v2 = v.clone();
    for x in v2.iter_mut() {
        *x += 1; // increment each element
    }
    println!("after iter_mut = {:?}", v2);
}

fn example_next() {
    println!("\n== Example 2: Using next() directly ==");
    let mut it = [1, 2, 3].iter();

    println!("next = {:?}", it.next()); // Some(&1)
    println!("next = {:?}", it.next()); // Some(&2)
    println!("next = {:?}", it.next()); // Some(&3)
    println!("next = {:?}", it.next()); // None (end)
}

fn example_adapters() {
    println!("\n== Example 3: Iterator adapters (lazy) ==");
    let nums = vec![1, 2, 3, 4, 5];

    // map squares
    let squares = nums.iter().map(|x| x * x);
    println!("squares = {:?}", squares.collect::<Vec<_>>());

    // filter evens
    let evens: Vec<_> = nums.iter().cloned().filter(|x| x % 2 == 0).collect();
    println!("evens = {:?}", evens);

    // chaining adapters
    let odds_squared: Vec<_> = nums.iter()
        .cloned()
        .filter(|x| x % 2 == 1)
        .map(|x| x * x)
        .collect();
    println!("odds_squared = {:?}", odds_squared);
}

fn example_consumers() {
    println!("\n== Example 4: Consuming adapters ==");
    let nums = vec![1, 2, 3, 4];

    let sum: i32 = nums.iter().sum();
    println!("sum = {}", sum);

    let product: i32 = nums.iter().product();
    println!("product = {}", product);

    let found = nums.iter().find(|&&x| x > 2);
    println!("first >2 = {:?}", found);

    nums.iter().for_each(|x| println!("for_each prints {}", x));

    let folded = nums.iter().fold(0, |acc, x| acc + x);
    println!("fold = {}", folded);
}

fn example_custom_iterator() {
    println!("\n== Example 5: Custom iterator implementing Iterator trait ==");

    struct Counter { n: u32 }
    impl Iterator for Counter {
        type Item = u32;
        fn next(&mut self) -> Option<Self::Item> {
            if self.n < 5 {
                self.n += 1;
                Some(self.n)
            } else {
                None
            }
        }
    }

    let mut c = Counter { n: 0 };
    println!("manual next: {:?}", (0..6).map(|_| c.next()).collect::<Vec<_>>());

    // reuse in for loop
    for val in Counter { n: 0 } {
        println!("Counter yields {}", val);
    }
}

fn main() {
    example_basic();
    example_next();
    example_adapters();
    example_consumers();
    example_custom_iterator();
}

/*
Docs-style notes:

Iterator trait:
- type Item;
- fn next(&mut self) -> Option<Self::Item>;
  - Returns Some(item) until exhausted, then None.
- Default methods (map, filter, fold, etc.) are built on top of next().

Kinds of iteration over Vec<T>:
- into_iter()  -> takes ownership, yields T, consumes the vector
- iter()       -> borrows immutably, yields &T, vector remains usable
- iter_mut()   -> borrows mutably, yields &mut T, can modify elements in place

Iterator adapters (lazy, return a new iterator):
- map, filter, filter_map, enumerate, zip, chain, take, skip, etc.
- Do nothing until consumed.

Consuming adapters:
- collect, for_each, fold, sum, product, find, any, all, count, etc.
- Drive the iteration to completion.

Custom iterators:
- Implement Iterator by writing your own next().
- Once you have next(), you automatically get access to all the adapters.

Performance:
- Iterators are zero-cost abstractions (monomorphized).
- Compiler optimizes chains of adapters into efficient loops (fusion).

Comparison:
| Category         | Example method      | Notes                                      |
|------------------|---------------------|--------------------------------------------|
| Entry points     | into_iter, iter     | How iteration begins                       |
| Lazy adapters    | map, filter, zip    | Return new iterators, do nothing immediately|
| Consuming        | collect, sum, fold  | Actually run the iteration                 |
| Custom impl      | impl Iterator::next | Create your own iterator types             |

*/
