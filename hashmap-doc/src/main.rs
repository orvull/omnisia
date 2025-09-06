//! HashMap<K, V> in Rust — mini-docs + runnable examples
//!
//! TL;DR
//! - `HashMap<K, V>`: growable hash table on the heap, average O(1) insert/lookup/remove.
//! - Keys must implement `Eq + Hash`. Order of iteration is NOT stable.
//! - Prefer the `entry` API for “insert-or-update” without double lookups.
//! - For lookups with borrowed forms (e.g., `String` key, `&str` lookup) use `get::<Q>` patterns.

use std::collections::HashMap;
use std::hash::Hash;

fn ex_basics() {
    println!("== Basics ==");
    // create
    let mut m: HashMap<String, i32> = HashMap::new();
    m.insert("apples".into(), 3);
    m.insert("bananas".into(), 5);

    // read
    println!("apples? {:?}", m.get("apples")); // Some(&3)
    println!("or zero: {}", m.get("oranges").copied().unwrap_or(0));

    // update / overwrite
    m.insert("apples".into(), 7); // overwrites previous value
    println!("apples now = {:?}", m.get("apples"));

    // contains / len / is_empty
    println!("has bananas? {}", m.contains_key("bananas"));
    println!("len={}, is_empty? {}", m.len(), m.is_empty());
}

fn ex_borrowed_lookup() {
    println!("\n== Borrowed lookup (&str vs String) ==");
    let mut m: HashMap<String, usize> = HashMap::new();
    m.insert("alpha".to_string(), 1);

    // You can query with &str even though keys are String:
    // HashMap::get<Q: ?Sized>(&self, k: &Q)
    // where K: Borrow<Q>, Q: Hash + Eq
    let q: &str = "alpha";
    println!("m.get(\"alpha\") = {:?}", m.get(q)); // Some(&1)

    // get_mut for in-place update
    if let Some(v) = m.get_mut("alpha") {
        *v += 1;
    }
    println!("after get_mut: {:?}", m.get("alpha"));
}

fn ex_entry_api() {
    println!("\n== entry() API (insert-or-update without double lookup) ==");
    let mut counts: HashMap<String, usize> = HashMap::new();
    for w in ["a", "b", "a", "c", "a", "b"] {
        *counts.entry(w.to_string()).or_insert(0) += 1;
    }
    println!("word counts = {:?}", counts);

    // and_modify + or_insert pattern
    let mut settings: HashMap<&'static str, i32> = HashMap::new();
    settings.insert("volume", 5);
    settings.entry("volume").and_modify(|v| *v += 1).or_insert(10);
    settings.entry("brightness").and_modify(|v| *v += 1).or_insert(50);
    println!("settings = {:?}", settings);

    // try_insert (avoids overwriting; returns Result)
    let mut cfg: HashMap<&str, &str> = HashMap::new();
    cfg.insert("mode", "fast");
    match cfg.try_insert("mode", "safe") {
        Ok(_) => println!("inserted mode"),
        Err(e) => println!("key existed, old value = {}", e.entry.get()),
    }
    println!("cfg = {:?}", cfg);
}

fn ex_iteration() {
    println!("\n== Iteration (order is arbitrary) ==");
    let mut m = HashMap::from([("x", 1), ("y", 2), ("z", 3)]);
    // by reference
    for (k, v) in &m {
        println!("&  {k} => {v}");
    }
    // by mutable reference
    for v in m.values_mut() {
        *v *= 10;
    }
    println!("after values_mut: {:?}", m);
    // keys/values views
    println!("keys   = {:?}", m.keys().collect::<Vec<_>>());
    println!("values = {:?}", m.values().collect::<Vec<_>>());
}

fn ex_remove_clear_retain() {
    println!("\n== remove / remove_entry / retain / clear ==");
    let mut m = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);

    // remove returns Option<V>
    let b = m.remove("b");
    println!("removed b -> {:?}, map = {:?}", b, m);

    // remove_entry returns Option<(K, V)>
    // (requires owned key type; here &str literal is 'static so fine via reinsert)
    m.insert("b", 22);
    let e = m.remove_entry("b");
    println!("remove_entry b -> {:?}, map = {:?}", e, m);

    // retain keeps entries that satisfy predicate
    m.retain(|_k, v| *v % 2 == 1);
    println!("retain odd -> {:?}", m);

    // clear empties without freeing capacity
    m.clear();
    println!("cleared -> len={}, cap~stays", m.len());
}

fn ex_capacity_and_grow() {
    println!("\n== Capacity management ==");
    let mut m: HashMap<i32, i32> = HashMap::with_capacity(2);
    println!("cap (initial guess) ~2; len={}", m.len());
    for i in 0..10 {
        m.insert(i, i * i);
    }
    println!("len={}, (capacity grows automatically)", m.len());

    // reserve additional space (reduce future rehashes)
    m.reserve(100);
    println!("reserved more; len={}", m.len());

    // shrink_to_fit may reduce allocation (not guaranteed)
    m.shrink_to_fit();
    println!("shrink_to_fit called.");
}

fn ex_building_collect_merge() {
    println!("\n== Build / collect / merge ==");
    // from iterator of pairs
    let m1: HashMap<_, _> = [("a", 1), ("b", 2)].into_iter().collect();
    let m2: HashMap<_, _> = vec![("b", 20), ("c", 3)].into_iter().collect();
    println!("m1={:?}, m2={:?}", m1, m2);

    // extend/merge: later values overwrite same keys
    let mut merged = m1.clone();
    merged.extend(m2.clone()); // now "b" -> 20
    println!("merged (extend) -> {:?}", merged);

    // merge-with-logic using entry
    let mut merged2 = m1.clone();
    for (k, v) in m2 {
        merged2.entry(k).and_modify(|old| *old += v).or_insert(v);
    }
    println!("merged2 (sum on conflict) -> {:?}", merged2);
}

fn ex_fn_signatures_and_passing() {
    println!("\n== Passing maps to functions (borrow vs own) ==");

    // Read-only view: &HashMap<K, V>
    fn total<K: Eq + Hash>(m: &HashMap<K, i32>) -> i32 {
        m.values().sum()
    }

    // In-place edit: &mut HashMap<K, V>
    fn bump<K: Eq + Hash>(m: &mut HashMap<K, i32>, by: i32) {
        for v in m.values_mut() {
            *v += by;
        }
    }

    // Take ownership (e.g., to return or store)
    fn into_keys<K: Eq + Hash, V>(m: HashMap<K, V>) -> Vec<K> {
        m.into_keys().collect()
    }

    let mut m = HashMap::from([("a", 1), ("b", 2)]);
    println!("total(&m) = {}", total(&m));
    bump(&mut m, 5);
    println!("after bump: {:?}", m);
    let keys = into_keys(m.clone());
    println!("into_keys -> {:?}", keys);
}

fn ex_common_patterns() {
    println!("\n== Common patterns ==");
    // Frequency count
    let text = "ababa";
    let mut freq: HashMap<char, usize> = HashMap::new();
    for ch in text.chars() {
        *freq.entry(ch).or_insert(0) += 1;
    }
    println!("freq('{text}') = {:?}", freq);

    // Grouping by key
    let pairs = [("eu", "pl"), ("eu", "de"), ("us", "ny")];
    let mut groups: HashMap<&str, Vec<&str>> = HashMap::new();
    for (k, v) in pairs {
        groups.entry(k).or_default().push(v);
    }
    println!("groups = {:?}", groups);

    // LRU-ish bump (toy)
    let mut hits: HashMap<&str, u64> = HashMap::new();
    for key in ["a", "b", "a", "c", "a"] {
        *hits.entry(key).or_insert(0) += 1;
    }
    println!("hits = {:?}", hits);
}

fn main() {
    ex_basics();
    ex_borrowed_lookup();
    ex_entry_api();
    ex_iteration();
    ex_remove_clear_retain();
    ex_capacity_and_grow();
    ex_building_collect_merge();
    ex_fn_signatures_and_passing();
    ex_common_patterns();
}

/*
Docs-style notes:

WHAT IT IS
- `HashMap<K, V>`: hash table mapping keys to values with average O(1) insert/lookup/remove.
- Keys: require `Eq + Hash`. Values: any type.
- Type: `pub struct HashMap<K, V, S = RandomState>` where S is the hasher builder.

OWNERSHIP & BORROWING
- The map OWNS its keys and values.
- Lookups take `&self` and a borrowed key: `get<Q>(&self, k: &Q) -> Option<&V>`
  where `K: Borrow<Q>`, `Q: Hash + Eq`. This enables `String` keys looked up by `&str`.
- Mutating lookups: `get_mut`, `entry`.
- Pass `&HashMap<K, V>` for read-only, `&mut HashMap<K, V>` for in-place edits,
  or `HashMap<K, V>` to transfer ownership.

INSERT / UPDATE / UPSERT
- `insert(k, v)` -> Option<V> (old value if key existed).
- `entry(k)`:
  * `.or_insert(v)` / `.or_default()` — insert if absent, then &mut V.
  * `.and_modify(|v| ...)` — run only when present (combine with or_insert for upsert).
  * `.or_insert_with(|| ...)` — lazily construct default.
- `try_insert(k, v)` -> Result<(), OccupiedEntry> (no overwrite).

LOOKUPS
- `get(&k)` / `get_mut(&k)`, `contains_key(&k)`.
- Borrowed lookup pattern: `map.get::<str>("key")` (type inference usually enough).
- `values()`, `values_mut()`, `keys()`, `iter()`, `iter_mut()`.

REMOVAL
- `remove(&k) -> Option<V>` (returns value).
- `remove_entry(&k) -> Option<(K, V)>` (returns key + value).
- `retain(|k, v| ...)`, `clear()`.

BUILD / MERGE
- From iterators of `(K, V)`: `iter.collect::<HashMap<_, _>>()`, `HashMap::from([...])`.
- `extend(other_map)` — overwrites on duplicate keys.
- Merge with logic: loop over other and use `entry` to combine.

CAPACITY & PERF
- `with_capacity(n)` to preallocate; `reserve(additional)` to grow; `shrink_to_fit()`.
- Table grows automatically as you insert; growth may rehash/move buckets.
- Iteration order is arbitrary and may change as the table grows.
- Average O(1), worst-case O(n) (pathological hashing).

INTERNALS (mental model)
- Heap-allocated hash table (std uses a `hashbrown`-style implementation with robin-hood probing).
- Fields include a pointer to buckets, length, and metadata for capacity/hash builder.
- Load factor triggers rehash/growth to keep O(1) averages.
- Hasher: default `RandomState` (SipHash-like); type param `S: BuildHasher` allows custom hashers.

FUNCTION SIGNATURES (when designing APIs)
- Read-only:      `fn f<K: Eq + Hash, V>(m: &HashMap<K, V>) { ... }`
- Mutating:       `fn f<K: Eq + Hash, V>(m: &mut HashMap<K, V>) { ... }`
- Take ownership: `fn f<K: Eq + Hash, V>(m: HashMap<K, V>) -> ...`
- Accept “map-like” iterables: `fn f<I, K, V>(it: I) where I: IntoIterator<Item=(K,V)>`

WHEN NOT TO USE HASHMAP
- Need ordered iteration / range queries → use `BTreeMap`.
- Need stable insertion order → consider `indexmap::IndexMap` (external crate).

COMMON PITFALLS
- Assuming stable iteration order (it isn’t).
- Double lookups for upsert instead of `entry`.
- Holding references across operations that may rehash (keep borrows short).
*/
