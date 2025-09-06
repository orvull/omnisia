//! Memory layout & initialization in Rust — mini-docs + runnable examples
//!
//! Topics:
//!  1) MaybeUninit<T>: uninitialized memory, manual init, *zeroing is not init*, safe patterns
//!  2) ManuallyDrop<T>: suppress Drop (FFI buffers, unions); compare with mem::forget
//!  3) Niche optimization & NonZero*: how `Option<NonZeroUsize>` is one word; `Option<&T>` too
//!
//! Run: `cargo run`

use std::{
    mem::{self, ManuallyDrop, MaybeUninit, size_of},
    num::{NonZeroU8, NonZeroUsize},
    ptr,
};

/* ───────────────────────────── 1) MaybeUninit<T> ─────────────────────────────
`MaybeUninit<T>` lets you handle memory that is not (yet) initialized, without
immediately invoking UB. You *must* initialize every byte of a `T` before you
treat it as a `T`.

Key APIs you’ll typically use:
- `MaybeUninit::<T>::uninit()`                 // uninitialized slot
- `MaybeUninit::<T>::new(value)`               // initialized slot
- `slot.write(value)`                          // write without reading old
- `assume_init()`                              // turn into T (only if fully init!)
- Arrays: `MaybeUninit::uninit_array()` + `MaybeUninit::array_assume_init(...)`
*/

pub fn ex_maybeuninit_array() {
    println!("== 1a) MaybeUninit: initialize array element-by-element ==");
    const N: usize = 4;

    // Allocate uninitialized array of T
    let mut buf: [MaybeUninit<String>; N] = MaybeUninit::uninit_array();

    // Initialize each element *exactly once*
    for i in 0..N {
        let s = format!("item-{i}");
        buf[i].write(s);
    }

    // SAFETY: we wrote all elements; no panics in between → fully initialized
    let arr: [String; N] = unsafe { MaybeUninit::array_assume_init(buf) };
    println!("array = {:?}", arr);
}

pub fn ex_maybeuninit_out_param() {
    println!("\n== 1b) MaybeUninit: out-parameter pattern ==");
    // Pretend we call an FFI that writes into a provided slot.
    #[inline]
    unsafe fn produce_into(slot: *mut u32) {
        // Initialize without reading the old memory:
        ptr::write(slot, 0xABCD_FFFF);
    }

    let mut slot: MaybeUninit<u32> = MaybeUninit::uninit();
    unsafe {
        produce_into(slot.as_mut_ptr());
        let val: u32 = slot.assume_init(); // fully initialized by callee
        println!("produced = 0x{val:08X}");
    }
}

/* ────────────────── “Zeroing is not init” (when it is / isn’t) ──────────────────
- For *plain old data* (POD) where the all-zero bit pattern is a valid value (e.g., u32),
  `MaybeUninit::<u32>::zeroed().assume_init()` is fine.
- For types like `String`, `Vec<T>`, `Box<T>`, zero bytes are NOT a valid representation
  (would violate their invariants) → assuming init after zeroing is UB.

We’ll demonstrate “okay” vs “not okay” in comments + a safe example:
*/

pub fn ex_zeroing_note() {
    println!("\n== 1c) Zeroing: when it’s okay vs UB ==");
    // OK: integers/pointers where 0 is valid
    let x = unsafe { MaybeUninit::<u32>::zeroed().assume_init() };
    println!("zeroed u32 = {x}");

    // ❌ NEVER do this (UB):
    // let s = unsafe { MaybeUninit::<String>::zeroed().assume_init() };

    // If you need a default String, *construct* it:
    let s = String::new();
    println!("constructed String OK: {:?}", s);
}

/* Safe patterns with MaybeUninit:
- Build arrays of non-Copy / no-Default elements, then assume_init after fully filling.
- Use `.write(...)` to overwrite uninitialized / possibly-garbage bytes without reading them.
- If initialization can fail mid-way, use a guard to drop already-initialized elements before unwind.
  (Omitted here for brevity; see std docs for a drop guard pattern.)
*/

/* ───────────────────────────── 2) ManuallyDrop<T> ─────────────────────────────
Wrap a value to *suppress automatic Drop*. You can later:
- extract it (consuming) via `ManuallyDrop::into_inner` (no Drop called on the wrapper),
- or call `ManuallyDrop::drop(&mut x)` manually if/when you choose.

Use cases:
- FFI buffers whose ownership you transfer (avoid double-free),
- unions containing non-Copy fields,
- custom drop ordering.

Compare with `mem::forget`: that *leaks* the value permanently. `ManuallyDrop`
lets you control when/how to drop or extract it.
*/

pub fn ex_manuallydrop_basics() {
    println!("\n== 2) ManuallyDrop basics, vs mem::forget ==");
    #[derive(Debug)]
    struct Loud(String);
    impl Drop for Loud {
        fn drop(&mut self) { println!("Drop(Loud: {:?})", self.0); }
    }

    // Suppress automatic drop
    let m = ManuallyDrop::new(Loud("held".into()));
    println!("wrapped: {:?}", unsafe { &*(&*m as *const Loud) });

    // 2a) Extract the inner value without running Drop on the wrapper:
    let inner: Loud = unsafe { ManuallyDrop::into_inner(m) };
    println!("extracted {:?}", inner.0);
    // Drop will run here (on `inner`) at end of scope.

    // 2b) Forget permanently (leak) — not recommended unless you *intend* to leak:
    let leak = Loud("leaked".into());
    mem::forget(leak); // no Drop() call will happen for this one
}

/* A tiny FFI-flavored example: take ownership of a raw allocation and prevent double drop. */
pub fn ex_manuallydrop_ffi_style() {
    println!("\n== 2b) ManuallyDrop: ownership transfer (FFI style) ==");
    // Imagine we received a Box<T> from C and must take ownership exactly once:
    let p = Box::new(String::from("ffi-owned"));
    let raw = Box::into_raw(p);           // C gives us this pointer…

    // Wrap the would-be Box in ManuallyDrop so we can control drop vs extraction:
    let mut wrapper: ManuallyDrop<Box<String>> = ManuallyDrop::new(unsafe { Box::from_raw(raw) });

    // Decide to *extract* and keep ownership in safe Rust:
    let owned_box: Box<String> = unsafe { ManuallyDrop::into_inner(ptr::read(&*wrapper)) };
    // SAFETY: we read (copy) the ManuallyDrop<..> content by value, leaving a moved-from wrapper.
    // We must not drop `wrapper` now (it contains moved value). That’s okay: it’s on the stack.

    println!("owned_box = {:?}", owned_box);
    // Drop occurs once, here, when owned_box goes out of scope.
}

/* ───────────── 3) Niche optimization & NonZero* (and pointers) ─────────────
A “niche” is a bit-pattern that a type never uses. The compiler can pack an `Option<T>`
into the same size as `T` by using the niche to encode `None`.

Examples:
- `NonZeroUsize` never uses 0 → `Option<NonZeroUsize>` is one word (0 encodes None).
- `&T` pointers are never null → `Option<&T>` is one word (null encodes None).
- `Box<T>` is non-null → `Option<Box<T>>` is one word too.

This saves space and improves cache behavior without extra code.
*/

pub fn ex_niche_sizes() {
    println!("\n== 3) Niche sizes (Option<T> size vs T) ==");
    println!("usize                      = {}", size_of::<usize>());
    println!("Option<usize>              = {}", size_of::<Option<usize>>()); // often also one word on many platforms
    println!("NonZeroUsize               = {}", size_of::<NonZeroUsize>());
    println!("Option<NonZeroUsize>       = {}", size_of::<Option<NonZeroUsize>>()); // == usize

    println!("&u8                        = {}", size_of::<&u8>());
    println!("Option<&u8>                = {}", size_of::<Option<&u8>>()); // == pointer size

    println!("Box<u8>                    = {}", size_of::<Box<u8>>());
    println!("Option<Box<u8>>            = {}", size_of::<Option<Box<u8>>>()); // == pointer size

    println!("NonZeroU8                  = {}", size_of::<NonZeroU8>());
    println!("Option<NonZeroU8>          = {}", size_of::<Option<NonZeroU8>>()); // == 1 byte
}

/* Using NonZero with Option in APIs */
pub fn ex_nonzero_api() {
    println!("\n== 3b) NonZero: ergonomic Option payloads ==");
    fn next_id(prev: Option<NonZeroUsize>) -> Option<NonZeroUsize> {
        // “0 means none” compactly encoded
        Some(prev.map_or(NonZeroUsize::new(1).unwrap(), |nz| NonZeroUsize::new(nz.get() + 1).unwrap()))
    }
    let a = next_id(None).unwrap();
    let b = next_id(Some(a)).unwrap();
    println!("ids: {} -> {}", a.get(), b.get());
}


/* ───────────────────────────── Docs-style notes ─────────────────────────────

MAYBEUNINIT<T>
- Use when you need a `T`’s storage before it’s fully initialized.
- Correct patterns:
  * Allocate: `let mut x: MaybeUninit<T> = MaybeUninit::uninit();`
  * Write once: `x.write(value)` or via raw pointer `ptr::write(x.as_mut_ptr(), value)`.
  * After fully initializing, convert: `unsafe { x.assume_init() }`.
  * For arrays: `MaybeUninit::uninit_array()` + write each element + `array_assume_init(...)`.
- Do NOT read from uninitialized memory. Do NOT call `assume_init` unless *every* byte is valid for `T`.
- “Zeroing is not init” unless the all-zero bit pattern is valid for `T` (e.g., integers, some C POD).
  Never zero-init `String`, `Vec<T>`, `Box<T>`, etc.

MANUALLYDROP<T>
- Prevents automatic Drop. Useful for:
  * Transferring ownership across FFI boundaries (avoid double-free).
  * Unions with non-Copy fields (control when to drop the active field).
  * Custom drop order in complex structures.
- Ways to use:
  * `let m = ManuallyDrop::new(value);`  // no Drop at scope end
  * Extract: `let v = unsafe { ManuallyDrop::into_inner(m) };` (consumes `m`)
  * Drop now: `unsafe { ManuallyDrop::drop(&mut m) }`
- `mem::forget(value)` *leaks* the value forever (never drops). Prefer `ManuallyDrop` when you still want control.

NICHE OPTIMIZATION (size wins)
- `Option<&T>` / `Option<Box<T>>` / `Option<NonZero*>` are the same size as their non-Option counterparts.
  The compiler uses an invalid/unused representation (null or zero) to encode `None`.
- Practically: choose `Option<NonZeroUsize>` instead of `Option<usize>` when you semantically exclude zero,
  to guarantee the one-word layout and document the invariant.
- This optimization is automatic. No unsafe needed.

PITFALLS
- UB magnets: calling `assume_init` too early; zero-initializing non-zeroable types; reading uninit bytes.
- Mixing partial init with panics: if constructing a collection element-by-element, use a guard to drop
  already-initialized elements on early exit.
- With `ManuallyDrop`, be sure each value is dropped exactly once (or intentionally leaked).

CHEATSHEET
- Uninit array:        `let mut a: [MaybeUninit<T>; N] = MaybeUninit::uninit_array();`
- Write elements:      `a[i].write(val);`
- Finalize:            `let arr: [T; N] = unsafe { MaybeUninit::array_assume_init(a) };`
- Suppress drop:       `let m = ManuallyDrop::new(v);`
- Extract owned:       `let v = unsafe { ManuallyDrop::into_inner(m) };`
- One-word Option:     `Option<NonZeroUsize>`, `Option<&T>`, `Option<Box<T>>`

*/
