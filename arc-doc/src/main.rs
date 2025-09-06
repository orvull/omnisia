use arc_doc::{
    example_atomic_counter,
    example_basic,
    example_mutation_with_mutex,
    example_rwlock_readers_writers,
    example_try_unwrap,
    example_weak_to_avoid_cycles,
};

fn main() {
    example_basic();
    example_mutation_with_mutex();
    example_rwlock_readers_writers();
    example_atomic_counter();
    example_try_unwrap();
    example_weak_to_avoid_cycles();
}
