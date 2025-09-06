//! Declarative Macros in Rust — mini-docs + runnable examples
//!
//! TL;DR
//! - `macro_rules! name { (pattern) => { expansion } ... }`
//! - Match token patterns at *compile time* and expand into Rust code.
//! - Captures use *fragment specifiers* (`$x:expr`, `$i:ident`, `$tt:tt`, …).
//! - Repetition: `$( ... )*`, `+`, `?` (with optional separators like `,` or `;`).
//! - Macros are hygienic; identifiers in expansions don’t accidentally collide.
//! - `$crate` points to the defining crate (works across crate boundaries).
//!
//! This file demonstrates:
//!  1) Basics & syntax sugar
//!  2) Fragment specifiers you’ll actually use
//!  3) Repetitions, separators, optional trailing comma
//!  4) Overloading by pattern (macro arms) + dispatch tricks
//!  5) Counting arguments (no runtime cost)
//!  6) Container builders: `vec!` / `hashmap!`-style
//!  7) TT-muncher recursion (tiny DSL)
//!  8) Hygiene & `$crate`
//!  9) API design tips (at bottom)

use std::collections::HashMap;

/* ────────────────────────────── 1) BASICS ────────────────────────────── */

// Simple forwarding wrapper: forward to println! while preserving formatting.
macro_rules! mprintln {
    ($($arg:tt)*) => {
        println!($($arg)*)
    };
}

/* ───────────────── 2) FRAGMENT SPECIFIERS (common ones) ─────────────────
A few of the many specifiers:

- ident     → an identifier (e.g., foo, Bar)
- path      → a path (e.g., std::io::Result, crate::module::Type)
- ty        → a type (e.g., Option<i32>, &str)
- expr      → an expression (e.g., 1+2, some_call())
- pat       → a pattern (e.g., Some(x), 3..=9)
- stmt      → a statement
- block     → a block `{ ... }`
- item      → an item (fn, struct, impl, etc.)
- lifetime  → a lifetime 'a
- meta      → meta item inside attributes (e.g., path = "x")
- tt        → a single token tree (most general; building block for munchers)
*/

// Tiny demo: accept different fragments and print a tag at runtime.
macro_rules! show_kind {
    ($x:ident)    => { mprintln!("ident: {}", stringify!($x)); };
    ($x:path)     => { mprintln!("path:  {}", stringify!($x)); };
    ($x:ty)       => { mprintln!("type:  {}", stringify!($x)); };
    ($x:expr)     => { mprintln!("expr:  {:?}", ($x)); };
    ($x:pat)      => { mprintln!("pat:   {}", stringify!($x)); };
    ($b:block)    => { mprintln!("block: {}", stringify!($b)); };
    ($m:meta)     => { mprintln!("meta:  {}", stringify!($m)); };
    ($t:tt)       => { mprintln!("tt:    {}", stringify!($t)); };
}

/* ─────────────── 3) REPETITIONS, SEPARATORS, TRAILING COMMA ─────────────── */

// Collect comma-separated expressions into a Vec.
//  - `$( $x:expr ),*`   = zero-or-more, separated by commas
//  - `$(,)?`            = optional trailing comma
macro_rules! make_vec {
    ( $( $x:expr ),* $(,)? ) => {{
        let mut v = Vec::new();
        $(
            v.push($x);
        )*
        v
    }};
}

// Key=>value hashmap literal (like maplit::hashmap! but minimal).
macro_rules! make_map {
    ( $( $k:expr => $v:expr ),* $(,)? ) => {{
        let mut m = ::std::collections::HashMap::new();
        $(
            m.insert($k, $v);
        )*
        m
    }};
}

/* ──────────────── 4) OVERLOADING BY PATTERN (macro arms) ──────────────── */

// Same macro name; different arms select by first token/shape.
macro_rules! over {
    // one expression
    ($x:expr) => { mprintln!("one expr = {:?}", $x); };
    // two expressions with comma
    ($a:expr, $b:expr) => { mprintln!("two exprs = {:?}, {:?}", $a, $b); };
    // named form: key = expr
    ($name:ident = $x:expr) => { mprintln!("named {} = {:?}", stringify!($name), $x); };
}

/* ───────────────────────── 5) COUNTING ARGUMENTS ─────────────────────────
Classic trick: map each argument to `()`, then take the array length *at compile time*.
We need a helper that replaces any token-tree with `()`.
*/
macro_rules! __replace_unit { ($_t:tt) => { () } }

macro_rules! count_args {
    ( $( $xs:tt ),* $(,)? ) => {
        <[()]>::len(&[ $( __replace_unit!($xs) ),* ])
    }
}

/* ────────────────────── 6) CONTAINER BUILDER EXAMPLES ────────────────────── */

// Re-implement a tiny subset of `vec!` (already in std, for teaching).
macro_rules! tiny_vec {
    // vec![elem; n] form
    ($elem:expr ; $n:expr) => {{
        let n = $n;
        let mut v = ::std::vec::Vec::with_capacity(n as usize);
        v.resize(n as usize, $elem);
        v
    }};
    // vec![a, b, c] form
    ( $( $x:expr ),* $(,)? ) => {{
        let mut v = ::std::vec::Vec::new();
        $( v.push($x); )*
        v
    }};
}

// HashMap builder with inferred types.
macro_rules! hashmap {
    ( $( $k:expr => $v:expr ),* $(,)? ) => {{
        let mut m = ::std::collections::HashMap::new();
        $( m.insert($k, $v); )*
        m
    }}
}

/* ──────────────────────── 7) TT-MUNCHER (recursive parse) ────────────────────────
We’ll parse a tiny "command list" DSL and produce code:
    cmds! { add 3; add 4; sub 1; }
expands to runtime code computing (((0 + 3) + 4) - 1).

Pattern: a recursive macro that "eats" tokens from the left until input is empty.
*/

macro_rules! cmds {
    // Entry point: start with accumulator = 0
    ( $($toks:tt)* ) => { cmds!(@acc 0 ; $($toks)* ) };

    // When input is empty -> yield the accumulator expr
    (@acc $acc:expr ; ) => { $acc };

    // Match `add <expr>; ...`
    (@acc $acc:expr ; add $x:expr ; $($rest:tt)* ) => {
        cmds!(@acc ($acc + ($x)) ; $($rest)* )
    };

    // Match `sub <expr>; ...`
    (@acc $acc:expr ; sub $x:expr ; $($rest:tt)* ) => {
        cmds!(@acc ($acc - ($x)) ; $($rest)* )
    };

    // Fallback: error if unknown token
    (@acc $acc:expr ; $bad:tt $($rest:tt)* ) => {
        compile_error!(concat!("cmds!: unexpected token: ", stringify!($bad)));
    };
}

/* ─────────────────────────── 8) HYGIENE & $crate ───────────────────────────
- Hygiene: identifiers introduced in the macro don’t accidentally capture or clash
  with variables at call-site.
- `$crate`: resolves to the defining crate, so paths work from dependents.
*/

// A "debug" macro that *creates a binding* internally (won’t clash with caller's).
#[macro_export] // pretend we export; `$crate` would point back here if this were a library
macro_rules! my_debug {
    ($e:expr) => {{
        // This `__val` is hygienic: distinct from any `__val` in caller code.
        let __val = &$e;
        $crate::mprintln!("[{}:{}] {} = {:?}", file!(), line!(), stringify!($e), __val);
        __val
    }};
}

/* ─────────────────────────────── EXAMPLES ─────────────────────────────── */

fn main() {
    mprintln!("== 1) basics");
    mprintln!("hello {}", "macros");

    mprintln!("\n== 2) fragment specifiers");
    show_kind!(foo);                     // ident
    show_kind!(std::collections::HashMap::<i32, i32>); // path
    show_kind!(Option<Result<i32, ()>>); // ty
    show_kind!(1 + 2 * 3);               // expr
    show_kind!({ let z = 1; z + 1 });    // block
    show_kind!(cfg(feature = "x"));      // meta
    show_kind!(<T as Into<U>>::into);    // tt (generic path)

    mprintln!("\n== 3) repetitions / separators / trailing comma");
    let a = make_vec![10, 20, 30,];
    mprintln!("make_vec -> {:?}", a);
    let m = make_map!{
        "a" => 1,
        "b" => 2,
    };
    mprintln!("make_map -> {:?}", m);

    mprintln!("\n== 4) overloading by pattern");
    over!(123);
    over!(10, 20);
    over!(answer = 42);

    mprintln!("\n== 5) count args");
    mprintln!("count() 0 => {}", count_args![]);
    mprintln!("count() 3 => {}", count_args![a, b, c]);
    mprintln!("count() 5 => {}", count_args![1, (x, y), {3}, foo, bar]);

    mprintln!("\n== 6) container builders");
    let v1 = tiny_vec![1, 2, 3];
    let v2 = tiny_vec![9; 4];
    mprintln!("tiny_vec lits     -> {:?}", v1);
    mprintln!("tiny_vec repeat   -> {:?}", v2);
    let hm: HashMap<&'static str, i32> = hashmap!{ "x" => 1, "y" => 2 };
    mprintln!("hashmap -> {:?}", hm);

    mprintln!("\n== 7) tt-muncher DSL");
    let result = cmds! { add 3; add 4; sub 1; add (2*2); };
    mprintln!("cmds! result = {}", result); // (((0+3)+4)-1)+(2*2) = 10

    mprintln!("\n== 8) hygiene & $crate");
    let __val = 999; // try to collide with internal name inside my_debug! (won't)
    let x = 123;
    let got = my_debug!(x * 2);
    mprintln!("my_debug returned {}", got);

    // Bonus: show that optional trailing commas are accepted
    let _ok = make_vec![ "a", "b", "c", ];
}

/* ────────────────────────────── DOCS NOTES ──────────────────────────────

MENTAL MODEL / “INTERNALS”
- Declarative macros are *compile-time* pattern matchers. They do not run at runtime and allocate no memory.
- The compiler tokenizes your source into token trees (TTs). `macro_rules!` matches those TTs against your arms.
- An arm that matches expands to tokens which are then parsed as Rust and compiled normally.
- Hygiene: new identifiers created inside the macro expansion don’t capture caller bindings (and vice versa).
- `$crate`: resolves to the crate where the macro is defined, so paths inside expansions remain correct when used from other crates.

COMMON FRAGMENT SPECIFIERS
- `$i:ident`, `$p:path`, `$t:ty`, `$e:expr`, `$pat:pat`, `$s:stmt`, `$b:block`, `$it:item`, `$l:lifetime`, `$m:meta`, `$tt:tt`.
- `$tt` is the most general; use it for recursive (tt-muncher) designs when other fragments are too restrictive.

REPETITIONS
- `$( PATTERN ),*`  → zero-or-more items separated by commas.
- `$( PATTERN ),+`  → one-or-more.
- `$( PATTERN )?`   → optional (0 or 1).
- Add separators like `;` or `,` between the parens.
- Optional trailing separator: append `$(,)?` or `$(;)?`.

OVERLOADING / DISPATCH
- Provide multiple arms ordered from most-specific to most-general; the first match wins.
- A catch-all arm with `$tt:tt` is handy for helpful `compile_error!`.

COUNTING ARGUMENTS (TRICK)
- Map each argument to `()` and measure slice length:
  `<[()]>::len(&[ $( __replace_unit!($x) ),* ])`.
- Useful for choosing different expansions based on arity (with `macro_rules!` recursion).

TT-MUNCHER PATTERN
- For simple DSLs, write a recursive macro:
  - Keep an accumulator (`@acc`) nonterminal.
  - Consume tokens left-to-right, transforming the accumulator.
  - End on empty input.

HYGIENE & `$crate`
- Don’t rely on caller’s local names; create your own bindings freely—they won’t clash.
- Use `$crate::path::to::item` inside exported macros so referenced items resolve from the def crate.

SCOPING / EXPORT
- Macros live in the module system. Invoke them after they’re visible (same module, `pub use`, or `#[macro_export]`).
- `#[macro_export]` places a macro at the crate root for downstream users; prefer re-exporting with `pub use` for namespacing.

DESIGN TIPS
- Keep expansions expression-based when possible: users can write `let x = mac!(...);`.
- Accept both with and without trailing comma: `$(,)?` improves ergonomics.
- For builders, prefer using fully qualified std paths in expansions (`::std::vec::Vec`) to avoid surprises.
- Provide helpful compile-time errors with `compile_error!(...)` on bad inputs.
- Avoid parsing *Rust* in macros you don’t need to—lean on fragments (`expr`, `ty`, `path`) rather than `tt` when possible.

LIMITATIONS
- Declarative macros can’t perform arbitrary computation; they manipulate tokens.
- No partial identifier construction on stable (avoid trying to “concatenate” idents—prefer `match`/traits/regular code).
- For advanced compile-time logic, consider `proc_macro` (procedural macros).

*/ 
