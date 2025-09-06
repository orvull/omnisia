//! Example consumer of the `macro_demo` procedural macros.

use macro_demo::{csv, HelloWorld, timeit};

#[derive(HelloWorld)]
struct User {
    id: u32,
    name: String,
}

// Time a (non-async) function
#[timeit]
fn heavy() -> u64 {
    // pretend heavy work
    let mut s = 0u64;
    for i in 0..50_000 {
        s = s.wrapping_add(i);
    }
    s
}

// Time with a custom label
#[timeit("custom label: compute()")]
fn compute(n: u64) -> u64 {
    (0..n).fold(0, |a, b| a.wrapping_add(b))
}

fn main() {
    println!("== derive(HelloWorld)");
    let u = User { id: 1, name: "Ada".into() };
    println!("{}", u.hello_world());

    println!("\n== attribute #[timeit]");
    let h = heavy();
    println!("heavy() -> {h}");
    let c = compute(100_000);
    println!("compute() -> {c}");

    println!("\n== function-like csv!(...)");
    // Turns token text into a compile-time concatenated &str
    let s = csv!(name, 1 + 2, some::path::<T>, "literal");
    println!("csv! => {}", s);

    // Empty list ⇒ empty string
    let empty = csv!();
    println!("csv!( ) => {:?}", empty);
}

/*
What you’ll see when you run:
- HelloWorld derive adds an inherent method: "Hello from User!"
- #[timeit] prints timing for heavy() and compute(...)
- csv!(...) prints the tokenized, comma-joined string at compile time
*/
