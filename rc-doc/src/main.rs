use rc::{
    example_basic,
    example_tree_like_sharing,
    example_mutation_with_refcell,
    example_weak_to_avoid_cycles,
};

fn main() {
    example_basic();
    example_tree_like_sharing();
    example_mutation_with_refcell();
    example_weak_to_avoid_cycles();
}
