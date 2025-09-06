//! Ownership Ergonomics in Rust — mini-docs + runnable examples
//!
//! Topics:
//!  1) `Cow<'a, T>` (copy-on-write) for “borrow most, own occasionally”; `ToOwned`
//!  2) Borrowing helpers: `Borrow`, `AsRef`, `Into`/`From` — flexible, zero-copy-ish APIs
//!  3) Guard types: `MutexGuard`, `RwLockReadGuard`/`RwLockWriteGuard`, `Ref`/`RefMut`
//!
//! Run: `cargo run`

use std::{
    borrow::{Borrow, Cow, ToOwned},
    cell::{RefCell, Ref, RefMut},
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
    thread,
    time::Duration,
};

/* ───────────────────────────── 1) Cow<'a, T> ─────────────────────────────
`Cow<'a, T>` = “Clone-On-Write”. It can be either:
- `Cow::Borrowed(&'a T)` → zero-copy borrow
- `Cow::Owned(T)`        → owned value you can mutate freely

When you *need to mutate* a borrowed cow, call `to_mut()` — it clones into owned first
(once), then you mutate the owned data. `Cow<T>` requires `T: ToOwned` (e.g., `str` ↔ `String`,
`[T]` ↔ `Vec<T>`).
*/

fn normalize_whitespace<'a>(input: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    let mut cow = input.into(); // may be borrowed or owned
    if cow.contains('\t') || cow.contains('\n') {
        // Need to modify: get mutable access; borrowed becomes owned here (clone-on-write).
        let s = cow.to_mut();
        *s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    cow
}

pub fn ex_cow_str() {
    println!("== 1) Cow<'a, str> (copy-on-write) ==");
    let borrowed: &str = "hello\tworld";
    let owned: String = "no-tabs-here".to_string();

    let a = normalize_whitespace(borrowed); // borrowed path → will clone because we modify
    let b = normalize_whitespace(&owned);   // borrowed path (from &String) → no change, stays borrowed
    let c = normalize_whitespace("a\nb\nc"); // &str → need to modify → becomes owned

    println!("a = {:?} (owned? {})", a, matches!(a, Cow::Owned(_)));
    println!("b = {:?} (borrowed? {})", b, matches!(b, Cow::Borrowed(_)));
    println!("c = {:?} (owned? {})", c, matches!(c, Cow::Owned(_)));
}

/* ───────────────── 1b) Cow for slices: &[T] ↔ Vec<T> ─────────────────
Useful when you usually pass a slice, but occasionally need to sort/unique/etc.
*/

fn sorted_unique<'a, T: Ord + Clone>(xs: impl Into<Cow<'a, [T]>>) -> Cow<'a, [T]> {
    let mut cow = xs.into();
    if !is_sorted_unique(&cow) {
        let v = cow.to_mut(); // clone to Vec<T> only if we need to change
        v.sort();
        v.dedup();
    }
    cow
}

fn is_sorted_unique<T: Ord>(slice: &[T]) -> bool {
    slice.windows(2).all(|w| w[0] < w[1])
}

pub fn ex_cow_slice() {
    println!("\n== 1b) Cow<'a, [T]> ==");
    let already_good = [1, 3, 5];
    let needs_work = vec![3, 1, 3, 2];

    let a = sorted_unique(&already_good[..]); // borrowed, unchanged
    let b = sorted_unique(&needs_work[..]);   // must own to sort/dedup

    println!("a: {:?} (borrowed? {})", a, matches!(a, Cow::Borrowed(_)));
    println!("b: {:?} (owned? {})", b, matches!(b, Cow::Owned(_)));
}

/* ─────────────────── 2) Borrow, AsRef, Into / From ───────────────────
Designing flexible APIs that accept many input types without copying.

- `AsRef<T>`: “cheap reference-to-reference conversion”. Great for read-only APIs.
  Examples: `AsRef<str>`, `AsRef<[u8]>`, `AsRef<Path>`.
- `Borrow<Q>`: like AsRef but preserves *Eq/Hash* semantics for key lookup (used by collections).
  Lets you look up `String` keys by `&str`, `PathBuf` by `&Path`, etc.
- `Into<T>` / `From<T>`: move/convert into an owned type. Prefer `impl Into<T>` to be flexible.
*/

fn sum_bytes<A: AsRef<[u8]>>(data: A) -> u64 {
    data.as_ref().iter().map(|&b| b as u64).sum()
}

fn print_path<P: AsRef<Path>>(p: P) {
    let path: &Path = p.as_ref();
    println!("path = {}", path.display());
}

// Lookup by borrowed key using Borrow<Q>
fn borrow_lookup_demo() {
    let mut map: HashMap<String, usize> = HashMap::new();
    map.insert("alpha".into(), 1);
    map.insert("beta".into(), 2);

    // Query with &str although map keys are String
    let k: &str = "alpha";
    // HashMap::get<Q: ?Sized>(&self, k: &Q) where String: Borrow<Q>, Q: Eq + Hash
    println!("get(\"alpha\") = {:?}", map.get(k)); // Some(&1)
}

// Owning conversion (Into/From) — commonly used when the callee needs to own the value.
fn needs_owned<S: Into<String>>(s: S) -> String {
    let owned: String = s.into();
    owned + "!"
}

pub fn ex_borrow_asref_into() {
    println!("\n== 2) Borrow / AsRef / Into ==");
    // AsRef examples
    println!("sum_bytes(&[1,2,3]) = {}", sum_bytes(&[1u8, 2, 3]));
    println!("sum_bytes(Vec)      = {}", sum_bytes(vec![4u8, 5, 6]));
    print_path("Cargo.toml");
    print_path(PathBuf::from("src/main.rs"));

    // Borrow for lookups
    borrow_lookup_demo();

    // Into for owned conversions
    println!("needs_owned(\"hi\") = {}", needs_owned("hi"));
    println!("needs_owned(String) = {}", needs_owned(String::from("yo")));
}

/* ────────────────────────── 3) Guard types ──────────────────────────
"Guards" are values that *own a lock or a borrow* and implement `Deref`/`DerefMut`
to access the protected inner value. When the guard is dropped, the lock/borrow is released.

- `MutexGuard<'a, T>`: holds exclusive lock on `Mutex<T>` for `'a`
- `RwLockReadGuard<'a, T>`: shared (read) lock on `RwLock<T>`
- `RwLockWriteGuard<'a, T>`: exclusive (write) lock on `RwLock<T>`
- `Ref<'a, T>` / `RefMut<'a, T>`: dynamic borrow guards from `RefCell<T>`

Key lifetime rule:
- You *cannot* return `&T` that outlives the guard; the reference is tied to the guard’s lifetime.
- Prefer returning an *owned* value (clone/copy) or confine use to the closure scope (“with_*” pattern).
*/

pub fn ex_mutex_guard_lifetimes() {
    println!("\n== 3a) MutexGuard lifetimes ==");
    let m = Mutex::new(String::from("secret"));

    // Scope the guard (keep lock short-lived):
    {
        let mut guard: MutexGuard<'_, String> = m.lock().unwrap();
        guard.push_str(" sauce");
        println!("inside lock: {}", *guard);
    } // guard dropped here → lock released

    // If you need data outside the lock, clone/move it out while holding the guard:
    let extracted: String = {
        let guard = m.lock().unwrap();
        guard.clone() // clone small amount of data; then guard drops
    };
    println!("outside lock (cloned) = {}", extracted);

    // Pattern: "with_lock" to restrict guard lifetime to a closure
    fn with_lock<T, R, F: FnOnce(&mut T) -> R>(m: &Mutex<T>, f: F) -> R {
        let mut g = m.lock().unwrap();
        f(&mut *g)
        // g drops here
    }
    let len = with_lock(&m, |s| s.len());
    println!("with_lock len = {}", len);
}

pub fn ex_rwlock_guards() {
    println!("\n== 3b) RwLock guards (many readers OR one writer) ==");
    let data = RwLock::new(vec![1, 2, 3]);

    // Many simultaneous readers
    {
        let r1: RwLockReadGuard<'_, Vec<i32>> = data.read().unwrap();
        let r2 = data.read().unwrap();
        println!("readers see: {:?} / {:?}", *r1, *r2);
        // r1, r2 drop here
    }

    // Exclusive writer
    {
        let mut w: RwLockWriteGuard<'_, Vec<i32>> = data.write().unwrap();
        w.push(4);
        println!("writer pushed: {:?}", *w);
    }

    // Another read
    println!("after write: {:?}", *data.read().unwrap());
}

pub fn ex_refcell_guards_runtime() {
    println!("\n== 3c) RefCell guards (Ref / RefMut) with runtime checks ==");
    let cell = RefCell::new(String::from("hi"));

    // Immutable borrow: multiple allowed
    let r1: Ref<'_, String> = cell.borrow();
    let r2 = cell.borrow();
    println!("r1={}, r2={}", r1.as_str(), r2.as_str());
    drop((r1, r2));

    // Mutable borrow: exclusive — panics at runtime if violated
    {
        let mut m: RefMut<'_, String> = cell.borrow_mut();
        m.push_str(" there");
        println!("mut = {}", m.as_str());
        // If we tried `let _r = cell.borrow();` here → panic!
    }
}

/* ─────────────────────────── 3d) Guard pitfalls ───────────────────────────
- Don’t hold a guard across slow IO / long computation → potential deadlocks/starvation.
- Don’t try to return `&T` from a function by derefing a guard; return owned or close over a closure.
- Avoid nested lock orders that can deadlock; standard trick: keep lock scopes small and consistent.
*/

pub fn ex_guard_pitfall_demo() {
    println!("\n== 3d) Guard pitfalls (quick demo) ==");
    let m1 = Arc::new(Mutex::new(0));
    let m2 = Arc::new(Mutex::new(0));

    // Bad: different lock order across threads can deadlock. We'll just *not* do it;
    // instead, lock in a consistent order (address-based).
    let a = m1.clone();
    let b = m2.clone();
    let t1 = thread::spawn(move || {
        lock_both_in_order(&a, &b, |x, y| { *x += 1; *y += 1; });
    });
    let t2 = thread::spawn(move || {
        lock_both_in_order(&m1, &m2, |x, y| { *x += 1; *y += 1; });
    });
    t1.join().unwrap();
    t2.join().unwrap();
    println!("m1={}, m2={}", *m1.lock().unwrap(), *m2.lock().unwrap());

    fn lock_both_in_order<T, F: FnOnce(&mut T, &mut T)>(
        a: &Mutex<T>,
        b: &Mutex<T>,
        f: F,
    ) {
        // Order by pointer address to avoid deadlock
        let (first, second) = if (a as *const _) < (b as *const _) { (a, b) } else { (b, a) };
        let mut g1 = first.lock().unwrap();
        let mut g2 = second.lock().unwrap();
        f(&mut *g1, &mut *g2);
        // guards drop in reverse order as they go out of scope
    }
}

/* ─────────────────────────────────── main ─────────────────────────────────── */


/* ───────────────────────────── Docs-style notes ─────────────────────────────

COW<'a, T>
- Type: `enum Cow<'a, B: ToOwned + ?Sized> { Borrowed(&'a B), Owned(<B as ToOwned>::Owned) }`
- Common aliases: `Cow<'a, str>` ↔ `String`, `Cow<'a, [T]>` ↔ `Vec<T>`.
- Use when your function *often* returns a borrow but *sometimes* needs to allocate or modify.
- Key methods: `Cow::Borrowed(_)/Owned(_)`, `into_owned()`, `to_mut()`, `is_borrowed()`/`is_owned()`.

BORROW / ASREF / INTO (and FROM)
- `AsRef<T>`: zero-cost ref conversion (borrow-in, borrow-out). Great for read-only params:
  `fn f<P: AsRef<Path>>(p: P) { let p: &Path = p.as_ref(); }`
- `Borrow<Q>`: like AsRef but preserves equality/hashing semantics — used in map/set lookups:
  `HashMap<String, V>::get(&str)` works because `String: Borrow<str>`.
- `Into<T>` / `From<T>`: owned conversions. Prefer `impl Into<T>` for parameters when the function
  must *take ownership*. At call sites, both `T` and types convertible *into* `T` are accepted.
  Blanket rule: `impl<T, U> Into<U> for T where U: From<T>` — if `From` exists, `Into` is auto-implemented.

GUARD TYPES (lifetimes and patterns)
- `MutexGuard<'a, T>`: holds the mutex lock until drop; deref to `&T` / `&mut T`. Don’t return `&T` that
  outlives the guard; clone/move data you need outside the lock or use a closure (“with_lock”).
- `RwLockReadGuard<'a, T>` / `RwLockWriteGuard<'a, T>`: many readers OR one writer; same lifetime rules.
- `Ref<'a, T>` / `RefMut<'a, T>`: runtime-checked borrows from `RefCell<T>`. Violations panic. Think of them
  as guards; keep them short-lived and don’t interleave conflicting borrows.

API DESIGN QUICK TIPS
- Read-only data → `&[T]`, `&str`, or generics `AsRef<[T]>`, `AsRef<str>`, `AsRef<Path>`.
- Owned result sometimes, borrowed otherwise → return `Cow<'a, T>`.
- Map lookups by borrowed form → use `Borrow<Q>` so users can pass `&str` for `String` keys, etc.
- Must own input → `impl Into<String>` (or a custom type), call `.into()` internally.
- Lock scope small; avoid holding guards across `.await` (in async) or slow IO / long compute sections.

COMMON PITFALLS
- Returning a reference derived from a guard — ties lifetime to the guard; either return owned or keep usage in the guard’s scope.
- Using `RefCell` across threads — it’s *not* `Sync`. Use `Mutex`/`RwLock` (or async variants) for multi-threading.
- Overusing `Into<String>` when you only need a `&str` — prefer `AsRef<str>` to avoid allocations.

CHEATSHEET
- Cow normalize:         `fn normalize<'a>(x: impl Into<Cow<'a, str>>) -> Cow<'a, str>`
- Read-only param:       `fn f<A: AsRef<[u8]>>(a: A)`
- Borrow lookup:         `map.get::<str>("key")` because `String: Borrow<str>`
- Own if needed:         `fn g<S: Into<String>>(s: S) { let s = s.into(); }`
- Mutex “with” pattern:  `fn with_lock<T,R,F:FnOnce(&mut T)->R>(m:&Mutex<T>, f:F)->R`
*/
