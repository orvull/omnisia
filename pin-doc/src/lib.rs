//! Pin<P> / Unpin in Rust — mini-docs + runnable examples
//!
//! TL;DR
//! - Pinning prevents *moves* of a value after it’s been pinned (its memory address must not change).
//! - `Unpin` (auto trait): types that are safe to move even when "pinned" (most types are `Unpin`).
//! - You pin *pointers*: `Pin<&mut T>`, `Pin<Box<T>>`, (and via APIs, `Pin<Rc<T>>` / `Pin<Arc<T>>`).
//! - You still can mutate pinned values; pinning restricts *relocation*, not *mutation*.
//! - Futures from `async fn` are usually `!Unpin`, so executors **pin** them before polling.
//!
//! This file demonstrates:
//!  1) Unpin basics and "pin as a no-op" when T: Unpin
//!  2) Pinning on the heap with `Box::pin` and address stability
//!  3) A `!Unpin` type via `PhantomPinned`: what you *can* and *cannot* do
//!  4) Safe & unsafe APIs on `Pin`: `get_ref`, `get_mut` (needs `Unpin`), `as_mut`, `map_unchecked_mut`
//!  5) Field projection basics (why it’s tricky) and a minimal, careful example
//!  6) Notes on async/futures and pinning
//!
//! Run with: `cargo run`

use std::marker::PhantomPinned;
use std::mem::{size_of, take};
use std::pin::Pin;
use std::ptr;

/// Pretty print an address (for demos)
fn addr_of<T>(r: &T) -> usize { r as *const T as usize }

/* ───────────────────────── 1) Unpin basics ─────────────────────────
`Unpin` means: "even if pinned, moving this value is harmless".
Most standard types are `Unpin` (e.g., integers, `String`, `Vec`, user structs
composed of `Unpin` fields). If T: Unpin, pinning is largely a *type-level* marker.
*/
pub fn ex_unpin_basics() {
    println!("== 1) Unpin basics ==");
    // i32 is Unpin; Pin<&mut i32> can be created and freely moved as a pointer wrapper.
    let mut x = 10i32;
    let mut pinned_ref: Pin<&mut i32> = Pin::new(&mut x);
    // Because i32: Unpin, we can get a &mut i32 back safely:
    let r: &mut i32 = Pin::get_mut(&mut pinned_ref);
    *r += 1;
    println!("x after Pin::get_mut = {}", x);

    println!("size_of::<Pin<&mut i32>>() = {}", size_of::<Pin<&mut i32>>());
}

/* ─────────────────── 2) Pin<Box<T>> and address stability ───────────────────
Heap-pin a value: moving the Pin<Box<T>> variable moves only the BOX (a pointer),
not the allocation containing T; T’s address remains stable.
*/
pub fn ex_box_pin_address_stability() {
    println!("\n== 2) Pin<Box<T>> address stability ==");
    let p = Box::pin(String::from("hello"));
    // Take the address of the *inner* String on the heap:
    let start_addr = addr_of(&*p);
    println!("inner addr at start = 0x{start_addr:x}");

    // Move the Pin<Box<String>> value around (ownership moves); inner address stays.
    let p = move_pin(p);
    let middle_addr = addr_of(&*p);
    println!("inner addr after move_pin() = 0x{middle_addr:x}");

    let p2 = p; // another move
    let end_addr = addr_of(&*p2);
    println!("inner addr after second move = 0x{end_addr:x}");

    assert_eq!(start_addr, middle_addr);
    assert_eq!(start_addr, end_addr);

    // We can still mutate the string contents while pinned (pin restricts *relocation*, not mutation):
    let mut p2 = p2;
    let mut_ref: Pin<&mut String> = Pin::as_mut(&mut p2);
    // Because String: Unpin, we may get &mut String back safely:
    let s: &mut String = Pin::get_mut(mut_ref);
    s.push_str(" world");
    println!("value = {}", s);
}
fn move_pin(p: Pin<Box<String>>) -> Pin<Box<String>> { p }

/* ───────────── 3) A !Unpin type with PhantomPinned ─────────────
If a type is `!Unpin` (does NOT implement Unpin), moving it after pinning is
*forbidden* (would be Undefined Behavior if you somehow did it). `PhantomPinned`
opts out of Unpin automatically.
*/
#[derive(Debug)]
struct SelfRef {
    data: String,
    // Imagine we want to hold a self-referential pointer/slice into `data`.
    // We won't actually create it (that requires careful construction), but
    // we mark the type as `!Unpin` so the compiler enforces pinning rules.
    _pin: PhantomPinned, // makes the type `!Unpin`
}

pub fn ex_non_unpin_type() {
    println!("\n== 3) !Unpin type with PhantomPinned ==");
    // Allocate on heap and pin:
    let mut s = Box::pin(SelfRef { data: String::from("abc"), _pin: PhantomPinned });

    // You can access &SelfRef:
    println!("pinned SelfRef.data = {}", s.data);

    // You may *mutate fields* through a pinned mutable reference (carefully):
    let mut s_pin_ref: Pin<&mut SelfRef> = Pin::as_mut(&mut s);
    // We cannot move `s`'s value out; but we can modify `data` in place:
    // To get &mut to a field, we must not move the whole struct. For Unpin fields,
    // we can use unsafe projection helpers (see next section). As a trivial safe demo:
    let new_data = take(&mut s_pin_ref.data); // `String` is Unpin; this replaces the field
    println!("took data (moved out field safely): {new_data}");
    // Put something back (still in-place field assignment):
    s_pin_ref.data = String::from("replaced");
    println!("now SelfRef.data = {}", s_pin_ref.data);

    // Because SelfRef is !Unpin, the following is illegal:
    // let moved = *s; // ❌ cannot move out (would require `SelfRef: Unpin`)
    // let inner = Pin::into_inner(s); // ❌ requires T: Unpin; SelfRef is !Unpin
}

/* ───────────── 4) Pin API: safe vs unsafe (and why) ─────────────
Key methods (selected):
- Pin::new(&mut T)            -> Pin<&mut T>              (safe)    // create pinned ref from &mut
- Box::pin(T)                 -> Pin<Box<T>>              (safe)    // allocate & pin on heap
- Pin::get_ref(&Pin<&T>)      -> &T                       (safe)    // read-only access
- Pin::as_mut(&mut Pin<P>)    -> Pin<&mut T>              (safe)    // reborrow as pinned &mut
- Pin::get_mut(&mut Pin<P>)   -> &mut T                   (safe IFF T: Unpin)
- Pin::into_inner(Pin<P>)     -> P::Target                (safe IFF T: Unpin)
- Pin::new_unchecked(...)     -> Pin<...>                 (unsafe)  // caller must uphold pin invariants
- map_unchecked_mut / map_unchecked   (unsafe)            // project fields (you must prove no move)
*/
#[derive(Debug)]
struct Container {
    a: String, // Unpin
    b: u64,    // Unpin
}
pub fn ex_pin_api_and_projection() {
    println!("\n== 4) Pin API & field projection (minimal) ==");
    let mut c = Box::pin(Container { a: "hi".to_string(), b: 7 });

    // Read-only access is easy & safe:
    println!("a={}, b={}", Pin::get_ref(&c).a, Pin::get_ref(&c).b);

    // Mutating through a pinned ref:
    // Step 1: get a `Pin<&mut Container>`
    let cref: Pin<&mut Container> = Pin::as_mut(&mut c);

    // If we want a pinned reference to a *field*, we must "project" without moving the outer struct.
    // The standard library doesn't auto-project; use crates (pin-project / pin-project-lite) in real code.
    // For Unpin fields, it's sound to produce an *unpinned* &mut:
    // SAFETY: We create an &mut to a field (`a`) without moving `Container`. That's fine.
    let a_mut: &mut String = unsafe { Pin::get_unchecked_mut(cref) }.a.as_mut();
    a_mut.push_str(" there");
    println!("after edit, a = {}", Pin::get_ref(&c).a);

    // If we needed a *pinned* projection (e.g., the field were `!Unpin`),
    // we'd need `map_unchecked_mut` + proof that the field's address won't change relative to `c`.
    // We won't do that here to keep things simple & safe.
}

/* ───────────── 5) Why field projection is hard (the short version) ─────────────
If `T: !Unpin`, pinning `Pin<&mut T>` promises the *whole T* will not move.
Projecting to a field and treating it as independently pinned requires proving that moving the
outer T cannot occur without also moving the field — which is why safe projection is nontrivial.
Crates like `pin-project` generate correct projections for you. Here we just explain the idea.
*/

/* ───────────── 6) Async & pinning (conceptual) ─────────────
- `async fn` returns an *anonymous* `impl Future<Output = T>` that is **usually `!Unpin`**.
- Executors (Tokio/etc.) **pin** futures before polling them: the state machine inside stores
  self-references between `.await` points, so its address must not change.
- If you manually poll a future, you typically do:
    let mut fut = my_async();          // impl Future (likely !Unpin)
    let mut fut = Box::pin(fut);       // Pin<Box<dyn Future>> or Pin<Box<_>>
    use std::future::Future;
    use std::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};
    // ... build a dummy Context and call fut.as_mut().poll(&mut cx)
- Most apps never need manual poll; runtimes handle pinning for you.
*/


/*
Docs-style notes:

WHAT PINNING GUARANTEES
- After a value is pinned, its memory location must not change (no "move").
- You pin pointer types (`&mut T`, `Box<T>`, etc.), not values directly.
- Moving the *pointer* (e.g., the Box itself) is OK; the allocation containing the value remains at a stable address.

`Unpin` (auto trait)
- If `T: Unpin`, it’s safe to move T even when pinned; pin is effectively a no-op for relocation.
- Most types are Unpin. Types that are self-referential (or want to prevent moves) are `!Unpin`.
- Opt out with `PhantomPinned` to make your type `!Unpin`.

HOW TO CREATE PINS
- Stack reference: `Pin::new(&mut t)` → `Pin<&mut T>` (valid for the borrow's lifetime).
- Heap allocation: `Box::pin(t)` → `Pin<Box<T>>` (common for long-lived / async cases).
- There are also APIs to pin in `Rc`/`Arc` on newer Rust versions; in practice, `Box::pin` is most common.

SAFE ACCESSORS
- `Pin::get_ref(&Pin<&T>) -> &T`                 // shared access
- `Pin::as_mut(&mut Pin<P>) -> Pin<&mut T>`       // reborrow as pinned &mut
- `Pin::get_mut(&mut Pin<P>) -> &mut T`           // only if T: Unpin
- `Pin::into_inner(Pin<P>) -> T`                  // only if T: Unpin

UNSAFE ACCESSORS (when you must prove “no move” yourself)
- `Pin::new_unchecked(ptr)`                       // create a Pin without checks
- `Pin::map_unchecked_mut`, `Pin::map_unchecked`  // project to fields
- Rule of thumb: prefer a projection macro crate (`pin-project`) over handwritten unsafe.

FIELD PROJECTION
- Safe projection is hard because pinning is about the *whole* value. If you pin a struct and
  want to pin an inner field, you must ensure the field can't outlive or move independently of the outer.
- Libraries provide safe generated projections—use them.

ASYNC CONNECTION
- Futures from `async fn` are typically `!Unpin`; executors pin them. This is why you often see `Pin<Box<dyn Future>>` internally.
- You rarely handle pinning explicitly in high-level async code; runtimes do it for you.

COMMON PITFALLS
- Thinking pinning prevents mutation—no, it prevents *relocation*. You can still mutate content.
- Using `get_mut`/`into_inner` on `!Unpin` types—won’t compile (that’s the point).
- Hand-rolling unsafe projection when you could use `pin-project(-lite)`.

CHEAT SHEET
- Pin a heap value:        `let p = Box::pin(val);`
- Reborrow pinned mutably: `let p_mut = p.as_mut();`
- Read fields:             `Pin::get_ref(&p).field`
- Mutate Unpin field:      `unsafe { Pin::get_unchecked_mut(p_mut) }.field = ...`
- Never move a `!Unpin` after pinning; keep borrows short & avoid mem::replace on the whole value.

*/
