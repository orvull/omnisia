use atomic_docs::{
    ex_acquire_release_flag,
    ex_atomic_cell_basics,
    ex_atomic_cell_threads,
    ex_atomic_ptr_and_fence,
    ex_compare_exchange,
    ex_relaxed_counter,
};

fn main() {
    ex_relaxed_counter();
    ex_acquire_release_flag();
    ex_compare_exchange();
    ex_atomic_ptr_and_fence();
    ex_atomic_cell_basics();
    ex_atomic_cell_threads();

    println!("\n== Cheatsheet (see comments below) ==");
}
