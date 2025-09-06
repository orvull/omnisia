//! Vectors & Slices in Rust — mini-docs + runnable examples
//!
//! TL;DR
//! - `Vec<T>`: growable, owned, contiguous buffer on the heap.
//! - `&[T]` / `&mut [T]`: non-owning *views* (slices) into contiguous memory.
//! - Prefer passing `&[T]` (read-only) or `&mut [T]` (in-place edit) to functions;
//!   pass `Vec<T>` only when the callee must own/resize it.
//! - Indexing panics OOB; prefer `.get()`/`.get_mut()` to avoid panics.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! INTERNAL REPRESENTATION (mental model)
//!
//! Vec<T> (roughly; generics elided):
//!   pub struct Vec<T> {
//!       ptr: NonNull<T>,  // pointer to heap buffer (may be dangling when cap=0)
//!       len: usize,       // number of initialized elements
//!       cap: usize,       // allocated capacity (in elements)
//!   }
//! - The *control block* (ptr/len/cap) lives on the stack; the *elements* live on the heap.
//! - Growing (push beyond capacity / reserve) may *reallocate* → moves the buffer.
//!   That invalidates raw pointers and any outstanding borrows/iters of the elements.
//!
//! Slice (&[T] / &mut [T]) — "fat" pointers:
//!   - A slice reference is (data_ptr: *const T, len: usize).
//!   - `&[T]` allows shared/read-only access; `&mut [T]` is exclusive and allows mutation.
//!   - Slices never own, never grow/shrink. Think “window into contiguous memory”.
//!
//! Coercions (automatic):
//!   &Vec<T>      → &[T]
//!   &mut Vec<T>  → &mut [T]
//!   Box<[T]>     → &[T]
//! - This is why function params should prefer `&[T]` / `&mut [T]`: callers can pass many
//!   container types without extra copies.
//!
//! Passing to functions (guidelines):
//!   - Read-only view         → fn f(xs: &[T])            // zero-copy, flexible
//!   - In-place modification  → fn f(xs: &mut [T])        // no reallocation
//!   - Take ownership/resize  → fn f(xs: Vec<T>)          // may reallocate / retain
//!   - Generic accept-many    → fn f<A: AsRef<[T]>>(a: A) // Vec, &[T], arrays, etc.
//!
//! Returning from functions:
//!   - Own the results        → Vec<T>
//!   - Exact-fit owned slice  → Box<[T]> (no spare capacity; use when you won’t grow)
//!
//! Size facts (on a 64-bit target):
//!   - size_of::<Vec<T>>()     == 3 * usize (ptr,len,cap)
//!   - size_of::<&[T]>()       == 2 * usize (ptr,len)     (same for &mut [T])
//!   - The elements’ size is *not* inside those control blocks.
//!
//! Memory/ABI notes:
//!   - Reallocation may move the buffer; keep borrows short around `push`/`reserve`.
//!   - Zero-sized types (ZSTs) like `()` have special handling (ptr may be dangling, len counts).
//!   - `into_boxed_slice()` can trim spare capacity and store tightly (good for long-lived data).

use std::mem::{size_of, size_of_val};

fn example_vec_basics() {
    println!("== Vec basics ==");
    let mut v1: Vec<i32> = Vec::new();
    v1.push(10);
    v1.push(20);

    let mut v2 = vec![1, 2, 3];    // literal macro
    let v3 = vec![0; 5];           // five zeros
    v2.extend([4, 5]);             // from array/iterator

    println!("v1 = {:?}", v1);
    println!("v2 = {:?}", v2);
    println!("v3 = {:?}", v3);

    println!("v2[0] = {}", v2[0]);                 // may panic if OOB
    println!("v2.get(100) = {:?}", v2.get(100));   // safe Option<&T>

    v2.insert(0, 99);              // O(n) shift right
    let popped = v2.pop();         // Option<T>, pop back (amortized O(1))
    v2.remove(1);                  // O(n) shift left
    v2.truncate(3);
    println!("v2 after edits = {:?}, popped = {:?}", v2, popped);
}

fn example_vec_capacity() {
    println!("\n== Capacity, reserve, shrink ==");
    let mut v = Vec::with_capacity(2);
    println!("start: len={}, cap={}", v.len(), v.capacity());

    v.extend([1, 2]);
    println!("after extend: len={}, cap={}", v.len(), v.capacity());

    v.reserve(100); // ensure extra space; may reallocate
    println!("after reserve: len={}, cap={}", v.len(), v.capacity());

    v.shrink_to_fit(); // may reduce capacity toward len
    println!("after shrink_to_fit: len={}, cap={}", v.len(), v.capacity());
}

fn example_vec_iterate() {
    println!("\n== Iterating Vec ==");
    let mut v = vec![10, 20, 30];

    for x in v.iter() {
        println!("iter saw {x}");  // &i32
    }

    for x in v.iter_mut() {
        *x += 1;                   // &mut i32
    }
    println!("after iter_mut: {:?}", v);

    for x in v.clone().into_iter() {
        println!("into_iter moved {x}"); // i32 by value
    }
}

fn example_vec_slice_views() {
    println!("\n== Slices from Vec ==");
    let mut v = vec![1, 2, 3, 4, 5, 6];

    let whole: &[i32] = &v;        // &Vec<T> → &[T] (coerce)
    let mid: &[i32]   = &v[2..4];  // half-open slice [2,4)
    println!("whole={:?}, mid={:?}", whole, mid);

    let tail: &mut [i32] = &mut v[3..];
    tail[0] = 99;                  // edits underlying Vec
    println!("after mut slice edit v={:?}", v);

    let owned_again: Vec<i32> = mid.to_vec(); // clone slice to owned
    println!("owned_again = {:?}", owned_again);
}

fn example_vec_batch_ops() {
    println!("\n== Batch ops: drain / retain / dedup / splice / split_off ==");
    let mut v = vec![1, 2, 2, 3, 3, 3, 4, 5];

    v.retain(|&x| x != 5);
    println!("retain !=5: {:?}", v);

    v.dedup();
    println!("dedup adjacent: {:?}", v);

    let drained: Vec<_> = v.splice(1..3, [20, 21, 22]).collect();
    println!("splice -> v={:?}, drained={:?}", v, drained);

    let mut w = vec![9, 8, 7, 6, 5];
    let d: Vec<_> = w.drain(1..4).collect();
    println!("drain 1..4 -> d={:?}, w now={:?}", d, w);

    let mut a = vec![1, 2, 3, 4, 5];
    let b = a.split_off(3); // a=[1,2,3], b=[4,5]
    println!("split_off -> a={:?}, b={:?}", a, b);
}

fn example_vec_sort_search() {
    println!("\n== Sort & binary_search ==");
    let mut v = vec![5, 1, 4, 2, 3];
    v.sort(); // stable
    println!("sorted = {:?}", v);

    let mut pairs = vec![("aa", 10), ("b", 2), ("ccc", 3)];
    pairs.sort_by_key(|&(s, _)| s.len());
    println!("sort_by_key(len) = {:?}", pairs);

    match v.binary_search(&4) {
        Ok(idx) => println!("found 4 at {idx}"),
        Err(ins) => println!("not found; insert at {ins}"),
    }
}

fn example_slice_basics() {
    println!("\n== Slice basics (&[T], &mut [T]) ==");
    let arr = [10, 20, 30, 40, 50];
    let s: &[i32] = &arr[1..4];
    println!("slice s = {:?}, len={}, first={}", s, s.len(), s[0]);

    println!("first()={:?}, last()={:?}", s.first(), s.last());
    println!("is_empty? {}", s.is_empty());

    let (left, right) = s.split_at(1);
    println!("split_at(1): left={:?}, right={:?}", left, right);

    let chunks: Vec<&[i32]> = arr.chunks(2).collect();
    let wins:   Vec<&[i32]> = arr.windows(3).collect();
    println!("chunks(2)={:?}", chunks);
    println!("windows(3)={:?}", wins);

    let mut arr2 = [3, 1, 2, 4];
    let s2: &mut [i32] = &mut arr2[..];
    s2.sort();
    println!("sorted slice -> arr2={:?}", arr2);

    let src = [9, 9, 9, 9];
    let dst = &mut arr2[..];
    dst.copy_from_slice(&src);
    println!("after copy_from_slice -> arr2={:?}", arr2);
}

fn example_slice_pattern_matching() {
    println!("\n== Slice pattern matching ==");
    let v = vec![1, 2, 3, 4, 5];

    match v.as_slice() {
        [] => println!("empty"),
        [x] => println!("one elem: {x}"),
        [x, y] => println!("two elems: {x},{y}"),
        [head, mid @ .., tail] => {
            println!("head={head}, tail={tail}, mid={:?}", mid);
        }
    }

    let mut w = vec![10, 20, 30, 40];
    match w.as_mut_slice() {
        [first, .., last] => {
            *first += 1;
            *last  += 1;
        }
        _ => {}
    }
    println!("after match-mutate: {:?}", w);
}

fn example_sizes_and_ptrs() {
    println!("\n== Sizes & pointers (64-bit targets) ==");
    let v = vec![1u64, 2, 3];
    let s: &[u64] = &v;

    println!("size_of::<Vec<u64>>()   = {}", size_of::<Vec<u64>>());
    println!("size_of::<&[u64]>()    = {}", size_of::<&[u64]>());
    println!("size_of_val(&v)        = {}", size_of_val(&v));
    println!("size_of_val(&s)        = {}", size_of_val(&s));

    println!("len={}, cap={}", v.len(), v.capacity());
    println!("vec.as_ptr() = {:p}", v.as_ptr());
    println!("slice.as_ptr() = {:p}", s.as_ptr()); // same data pointer
}

fn example_passing_to_functions() {
    println!("\n== Passing Vec/Slice to functions (ownership vs borrowing) ==");

    // Read-only: accept &[T]
    fn sum(xs: &[i32]) -> i32 {
        xs.iter().sum()
    }

    // In-place edit: accept &mut [T] (cannot grow)
    fn bump_all(xs: &mut [i32]) {
        for x in xs { *x += 1; }
    }

    // Take ownership: accept Vec<T> (may grow/shrink/keep)
    fn into_even_only(mut xs: Vec<i32>) -> Vec<i32> {
        xs.retain(|x| x % 2 == 0);
        xs
    }

    // Generic accept-many: AsRef<[T]> (Vec, &[T], arrays, Box<[T]>, etc.)
    fn print_all<A: AsRef<[i32]>>(a: A) {
        for x in a.asRef() {
            print!("{x} ");
        }
        println!();
    }

    // — demo —
    let mut v = vec![1, 2, 3, 4, 5];
    println!("sum(&v) = {}", sum(&v)); // &Vec<T> → &[T]

    bump_all(&mut v); // &mut Vec<T> → &mut [T]
    println!("after bump_all: {:?}", v);

    let evens = into_even_only(v.clone()); // moves (clones to keep original here)
    println!("evens from into_even_only: {:?}", evens);

    print!("print_all(Vec): ");
    print_all(v.clone());
    print!("print_all(slice): ");
    print_all(&v[..]);
    print!("print_all(array): ");
    print_all([7, 8, 9]);
}

fn example_boxed_slice_return() {
    println!("\n== Returning Box<[T]> (tight, no spare capacity) ==");
    fn to_boxed(mut xs: Vec<i32>) -> Box<[i32]> {
        xs.shrink_to_fit();          // try to remove spare capacity
        xs.into_boxed_slice()        // owned slice (exact len)
    }

    let b = to_boxed(vec![1, 2, 3, 4]);
    println!("boxed slice len={}, first={}", b.len(), b[0]);
}

fn example_safety_and_panic_free() {
    println!("\n== Safety tips: indexing vs get, reallocation ==");
    let mut v = vec![1, 2, 3];

    if let Some(x) = v.get(100) {
        println!("unexpected: {x}");
    } else {
        println!("safe: index 100 is out of bounds");
    }

    let (left, right) = v.split_at_mut(1);
    left[0] += 10;
    if let Some(r0) = right.get_mut(0) {
        *r0 += 20;
    }
    println!("after split_at_mut edits: {:?}", v);

    // Reallocation demo (raw pointer invalidation)
    let p = v.as_ptr();
    let old_cap = v.capacity();
    v.reserve(10_000); // likely reallocate
    println!("ptr changed? {} -> {}", format!("{:p}", p), format!("{:p}", v.as_ptr()));
    println!("cap {} -> {}", old_cap, v.capacity());
}

fn main() {
    example_vec_basics();
    example_vec_capacity();
    example_vec_iterate();
    example_vec_slice_views();
    example_vec_batch_ops();
    example_vec_sort_search();
    example_slice_basics();
    example_slice_pattern_matching();
    example_sizes_and_ptrs();
    example_passing_to_functions();
    example_boxed_slice_return();
    example_safety_and_panic_free();
}

/*
Docs-style notes (expanded):

INTERNALS
- Vec<T> is a tiny stack object with three words: (ptr, len, cap). Elements live on the heap.
- Slices &[T]/&mut [T] are *fat* references: (data_ptr, len). They never own and can’t resize.
- Reallocation moves the heap buffer. Keep borrows/iterators short around push/reserve.

PASSING TO FUNCTIONS (choose the lightest that fits)
- Read-only, most flexible:              fn f(xs: &[T])
  Callers can pass &Vec<T>, arrays, Box<[T]>, other slices, without copies.
- In-place edit, fixed-size window:      fn f(xs: &mut [T])
  Callee may mutate, but cannot grow/shrink the buffer.
- Needs ownership or resizing:           fn f(xs: Vec<T>)
  Callee can reserve/push/pop/retain and keep or return it.
- Accept “anything slice-like”:          fn f<A: AsRef<[T]>>(a: A)
  Great for libraries; supports Vec, &[T], Box<[T]>, arrays, Cow<[T]>, etc.

RETURN TYPES
- Return `Vec<T>` when you’re handing back *owned growable data*.
- Return `Box<[T]>` when you want *owned, exact-sized* data with no spare capacity
  (smaller footprint, can be more cache-friendly for read-only blobs).

COERCIONS
- &Vec<T>      → &[T]
- &mut Vec<T>  → &mut [T]
- Box<[T]>     → &[T]
- Many std APIs accept slices for maximum flexibility and zero-copy interop.

SAFETY/PERF TIPS
- Prefer `get()`/`get_mut()` when indices may be invalid; indexing panics on OOB.
- Use iterators (`iter`, `iter_mut`, adapters) for clarity and bounds-checked, fused loops.
- Sorting/search: `sort`, `sort_by_key`, `binary_search` (requires sorted input).
- Batch transforms: `retain`, `drain`, `splice`, `split_off` avoid repeated reallocations.
- Avoid holding references across potential reallocation points (`push`, `reserve`, `append`).

ADVANCED
- `into_boxed_slice()` trims spare capacity (Vec → Box<[T]>) for tight storage.
- Zero-sized types (ZSTs) are supported; ptr may be “dangling”, length still meaningful.
- FFI often prefers slices as (ptr,len) pairs; `as_ptr()` and `len()` provide those.

*/
