use pattern_matchine_docs::{
    ex_match_basics,
    ex_tuple_struct_enum,
    ex_option_result,
    ex_guards_bindings_ranges,
    ex_slice_patterns,
    ex_references_boxes,
    ex_while_let,
    ex_matches_macro,
    ex_ignore_parts,
    ex_shadowing_and_order,
    ex_function_param_patterns,
};

fn main() {
    ex_match_basics(4);
    ex_tuple_struct_enum();
    ex_option_result();
    ex_guards_bindings_ranges(2);
    ex_slice_patterns();
    ex_references_boxes();
    ex_while_let();
    ex_matches_macro();
    ex_ignore_parts();
    ex_shadowing_and_order();
    ex_function_param_patterns();
}
