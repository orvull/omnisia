//! Pattern Matching in Rust — mini-docs + runnable examples
//!
//! Patterns let you concisely decompose and test data shapes in `match`, `let`, `if let`,
//! `while let`, function params, and more. They’re exhaustive by default in `match`.

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
    email: Option<String>,
}

#[derive(Debug)]
enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Unit,                    // unit-like
}

pub fn ex_match_basics(x: i32) {
    println!("== match basics ==");
    let msg = match x {
        0 => "zero",
        1 | 2 => "one or two",                 // alternatives
        3..=9 => "three to nine (inclusive)",  // range
        _ => "something else",                 // wildcard (exhaustiveness)
    };
    println!("x={x} -> {msg}");
}

pub fn ex_tuple_struct_enum() {
    println!("\n== tuples, structs, enums ==");
    let pair = (1, "hi", true);

    // tuple destructuring in `let` is a pattern as well
    let (a, b, c) = pair;
    println!("tuple destructured: a={a}, b={b}, c={c}");

    // struct destructuring (with `..` to ignore the rest)
    let user = User { id: 42, name: "Roman".into(), email: Some("r@example.com".into()) };
    let User { id, name, .. } = &user; // borrow destructure
    println!("User fields (borrowed): id={id}, name={name}");

    // enum matching
    let s = Shape::Rect { w: 3.0, h: 5.0 };
    match s {
        Shape::Circle { r } => println!("circle r={r}"),
        Shape::Rect { w, h } => println!("rect w={w}, h={h}"),
        Shape::Unit => println!("unit"),
    }
}

pub fn ex_option_result() {
    println!("\n== Option / Result ==");
    let maybe: Option<i32> = Some(10);
    match maybe {
        Some(v) if v % 2 == 0 => println!("even Some({v})"), // guard
        Some(v) => println!("odd Some({v})"),
        None => println!("None"),
    }

    let res: Result<&str, &str> = Err("boom");
    match res {
        Ok(v) => println!("Ok: {v}"),
        Err(e) => println!("Err: {e}"),
    }

    // if let sugar for single-arm matches
    if let Some(v) = maybe {
        println!("if let got {v}");
    }
}

pub fn ex_guards_bindings_ranges(x: i32) {
    println!("\n== guards, @ bindings, ranges ==");
    match x {
        small @ -3..=3 => println!("small or near zero: {small}"), // bind + test range
        n if n % 2 == 0 => println!("even by guard: {n}"),         // guard after pattern
        _ => println!("other"),
    }
}

pub fn ex_slice_patterns() {
    println!("\n== slice patterns ==");
    let data = [10, 20, 30, 40];

    match data {
        // fixed-size binding with prefix/suffix + rest (`..`) in the middle
        [first, .., last] => println!("first={first}, last={last}"),
    }

    let v = vec![1, 2, 3, 4, 5];
    match v.as_slice() {
        [] => println!("empty"),
        [x] => println!("one element: {x}"),
        [x, y] => println!("two elements: {x},{y}"),
        [head, mid @ .., tail] => {
            println!("head={head}, tail={tail}, mid={:?}", mid);
        }
    }
}

pub fn ex_references_boxes() {
    println!("\n== references & boxes ==");
    let n = 10;
    let r = &n;
    match r {
        // match on reference: pattern `&pat` peels one layer of reference
        &val if val > 5 => println!("ref > 5: {val}"),
        &val => println!("ref other: {val}"),
    }

    let b = Box::new(String::from("hello"));
    match b {
        // `box` pattern moves out of the Box (value owned here)
        box s => println!("boxed string moved out: {s}"),
    }
}

pub fn ex_while_let() {
    println!("\n== while let ==");
    let mut it = (1..=3).peekable();
    // consume while pattern matches
    while let Some(&next) = it.peek() {
        println!("peek={next}");
        it.next();
    }
}

pub fn ex_matches_macro() {
    println!("\n== matches! macro ==");
    let s = Shape::Circle { r: 2.0 };
    if matches!(s, Shape::Circle { .. }) {
        println!("it is a circle!");
    }
}

pub fn ex_ignore_parts() {
    println!("\n== ignoring with _ and .. ==");
    let user = User { id: 7, name: "Neo".into(), email: None };

    match user {
        User { id, .. } => println!("only care about id={id}"), // ignore others
    }

    // ignore some tuple parts
    let coords = (3, 4, 5);
    match coords {
        (x, _, z) => println!("x={x}, z={z}"),
    }
}

pub fn ex_shadowing_and_order() {
    println!("\n== binding shadowing & arm order ==");
    let x = 5;

    match x {
        1 | 2 => println!("one or two"),
        // order matters: this guard arm runs before wildcard
        n if n > 3 => println!(">3 via guard: {n}"),
        _ => println!("something else"),
    }

    // shadowing pattern variables (separate from outer x)
    let x_outer = 10;
    match 42 {
        x @ 40..=50 => println!("bound x={x} (shadows x_outer={x_outer})"),
        _ => {}
    }
}

pub fn ex_function_param_patterns() {
    println!("\n== patterns in function params ==");
    // destructure in params
    fn sum_pair((a, b): (i32, i32)) -> i32 { a + b }
    println!("sum_pair = {}", sum_pair((3, 4)));

    // destructure struct param partially
    fn show_user(User { id, name, .. }: &User) {
        println!("User(id={id}, name={name})");
    }
    let u = User { id: 1, name: "Ada".into(), email: None };
    show_user(&u);
}


/*
Docs-style notes:

Pattern positions:
- `match expr { pat => ... }`
- `let pat = expr;`
- `if let pat = expr { ... }` / `while let pat = expr { ... }`
- Function parameters: `fn f((a, b): (i32, i32))`
- Closures too: `|User { id, .. }| ...`

Common pattern tools:
- `_`          : wildcard, ignore the value
- `..`         : ignore “the rest” (structs, tuples, slices)
- `|`          : alternatives (OR patterns)
- Ranges       : `1..=5`, `'a'..='z'` (only with integer/char)
- Guards       : `pat if condition`
- `@` binding  : bind matched value while testing its shape (e.g., `n @ 0..=9`)
- `&` / `&mut` : reference patterns peel ref layers (e.g., `&x`, `&mut y`)
- `box`        : box pattern to move out of `Box<T>` (ownership transferred)

Exhaustiveness:
- `match` must be exhaustive. Add `_ => ...` or cover all variants.
- Arm order matters; the first matching arm runs.

Matching ergonomics:
- Matching on references often auto-derefs; use `&pat` to bind by value of a reference.
- Use `ref`/`ref mut` in older code; modern Rust prefers `&` / `&mut` patterns.

Option/Result sugar:
- `if let Some(x) = opt { ... }` for single-interest cases.
- `while let Some(x) = iter.next() { ... }` to consume iterators.

Slices:
- Array/slice patterns support `[a, b]`, `[head, ..]`, `[.., tail]`, `[h, mid @ .., t]`.
- `mid @ ..` binds the “rest” as a subslice.

Ignoring:
- `_` discards a single value; `..` discards multiple/remaining fields/elements.
- Combine with explicit bindings to keep just what you need.

Bindings:
- `@` lets you both test a shape and keep the original: `n @ 1..=9`.

Performance:
- Patterns are zero-cost; the compiler generates optimal tests/binds.
- Guards run only after the structural pattern matches.

*/
