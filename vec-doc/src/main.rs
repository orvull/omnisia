use vec_doc::{
    example_vec_basics,
    example_vec_capacity,
    example_vec_iterate,
    example_vec_slice_views,
    example_vec_batch_ops,
    example_vec_sort_search,
    example_slice_basics,
    example_slice_pattern_matching,
    example_sizes_and_ptrs,
    example_passing_to_functions,
    example_boxed_slice_return,
    example_safety_and_panic_free,
};

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
