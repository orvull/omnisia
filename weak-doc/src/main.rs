use weak_doc::{
    ex_rc_weak_basics,
    ex_rc_cycle_vs_weak,
    ex_arc_weak_multithread,
    ex_cache_with_weak,
    ex_leak_then_fix,
};

fn main() {
    ex_rc_weak_basics();
    ex_rc_cycle_vs_weak();
    ex_arc_weak_multithread();
    ex_cache_with_weak();
    ex_leak_then_fix();
}
