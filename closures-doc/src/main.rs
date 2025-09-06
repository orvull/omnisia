//! Closures in Rust: extended docs + examples
//!
//! Internally, closures are basically structs that hold captured variables
//! and implement one (or more) of the traits: Fn, FnMut, FnOnce.

fn example_basic() {
    println!("== Example 1: Basic closure ==");
    let add_one = |x: i32| x + 1;
    println!("3 + 1 = {}", add_one(3));

    // Rough internal equivalent:
    struct AddOne;
    impl AddOne {
        fn call(&self, x: i32) -> i32 { x + 1 }
    }
    let add_one_struct = AddOne;
    println!("3 + 1 (struct) = {}", add_one_struct.call(3));
}

fn example_capture_by_ref() {
    println!("\n== Example 2: Capture by reference (&T) ==");
    let x = 10;
    let print_x = || println!("x = {}", x);
    print_x();
    print_x();

    // Rough internal equivalent:
    struct PrintX<'a> { x_ref: &'a i32 }
    impl<'a> PrintX<'a> {
        fn call(&self) { println!("x = {}", self.x_ref); }
    }
    let print_x_struct = PrintX { x_ref: &x };
    print_x_struct.call();
}

fn example_capture_by_mut() {
    println!("\n== Example 3: Capture by mutable reference (&mut T) ==");
    let mut y = 0;
    let mut inc_y = || { y += 1; println!("y = {}", y); };
    inc_y();
    inc_y();

    // Rough internal equivalent:
    struct IncY<'a> { y_ref: &'a mut i32 }
    impl<'a> IncY<'a> {
        fn call(&mut self) {
            *self.y_ref += 1;
            println!("y = {}", self.y_ref);
        }
    }
    let mut y2 = 0;
    let mut inc_y_struct = IncY { y_ref: &mut y2 };
    inc_y_struct.call();
    inc_y_struct.call();
}

fn example_capture_by_move() {
    println!("\n== Example 4: Capture by move (T, owned) ==");
    let s = String::from("owned");
    let consume_s = move || println!("consumed: {}", s);
    consume_s();

    // Rough internal equivalent:
    struct ConsumeS { s: String }
    impl ConsumeS {
        fn call_once(self) { println!("consumed: {}", self.s); }
    }
    let consume_s_struct = ConsumeS { s: String::from("owned2") };
    consume_s_struct.call_once(); // can only call once
}

fn example_fn_traits() {
    println!("\n== Example 5: Fn, FnMut, FnOnce traits ==");
    fn call_fn<F: Fn()>(f: F) { f(); }
    fn call_fn_mut<F: FnMut()>(mut f: F) { f(); }
    fn call_fn_once<F: FnOnce()>(f: F) { f(); }

    let x = 5;
    let closure_fn = || println!("Fn sees x = {}", x);
    call_fn(closure_fn);

    let mut y = 0;
    let closure_fnmut = || { y += 1; println!("FnMut increments y = {}", y); };
    call_fn_mut(closure_fnmut);

    let s = String::from("take");
    let closure_fnonce = move || println!("FnOnce takes s = {}", s);
    call_fn_once(closure_fnonce);
}

fn example_returning_closure() {
    println!("\n== Example 6: Returning closures ==");
    fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
        move |x| x + n // capture n by value
    }
    let add10 = make_adder(10);
    println!("add10(5) = {}", add10(5));

    // Rough internal equivalent:
    struct Adder { n: i32 }
    impl Adder {
        fn call(&self, x: i32) -> i32 { x + self.n }
    }
    let adder_struct = Adder { n: 10 };
    println!("adder_struct.call(5) = {}", adder_struct.call(5));
}

fn example_iterators() {
    println!("\n== Example 7: Closures in iterators ==");
    let nums = vec![1, 2, 3, 4];

    let squares: Vec<_> = nums.iter().map(|x| x * x).collect();
    println!("squares = {:?}", squares);

    // Equivalent: Map holds a closure struct internally
    struct Square;
    impl Square {
        fn call(&self, x: &i32) -> i32 { x * x }
    }
    let squares2: Vec<_> = vec![1, 2, 3, 4].iter().map(|x| Square.call(x)).collect();
    println!("squares (manual struct) = {:?}", squares2);
}

fn main() {
    example_basic();
    example_capture_by_ref();
    example_capture_by_mut();
    example_capture_by_move();
    example_fn_traits();
    example_returning_closure();
    example_iterators();
}

/*
Docs-style notes:

How closures are built internally:
- Each closure becomes an anonymous compiler-generated struct.
- Captured variables become fields in that struct.
- The struct implements one or more traits: Fn, FnMut, FnOnce.
- Which trait is implemented depends on how captures happen:
  * Capture by &T       -> implements Fn
  * Capture by &mut T   -> implements FnMut (and FnOnce)
  * Capture by move (T) -> implements FnOnce (and maybe FnMut/Fn if Copy)
- Calling the closure is just calling the `call` method on that hidden struct.

Examples:
| Closure                   | Internal struct-like form             | Trait bound |
|----------------------------|---------------------------------------|-------------|
| let c = || println!(x);    | struct C { x_ref: &i32 }              | Fn          |
| let mut c = || { y += 1; } | struct C { y_ref: &mut i32 }          | FnMut       |
| let c = move || println!(s); | struct C { s: String } (owned)      | FnOnce      |

Closures vs functions:
- Functions: fixed type, no captures.
- Closures: may capture environment, so type is unique and inferred.
- Both can be used where Fn traits are expected.

Performance:
- Zero-cost abstraction: closure structs are monomorphized like generics.
- No runtime overhead compared to writing the struct manually.

*/
