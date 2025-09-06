use ownership_egro_doc::{
    ex_cow_str,
    ex_cow_slice,
    ex_borrow_asref_into,
    ex_mutex_guard_lifetimes,
    ex_rwlock_guards,
    ex_refcell_guards_runtime,
    ex_guard_pitfall_demo,
};

fn main() {
    ex_cow_str();
    ex_cow_slice();
    ex_borrow_asref_into();
    ex_mutex_guard_lifetimes();
    ex_rwlock_guards();
    ex_refcell_guards_runtime();
    ex_guard_pitfall_demo();
    println!("\n== Cheatsheet in comments below ==");
}
