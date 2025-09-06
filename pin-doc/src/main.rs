use pin_doc::{
    ex_unpin_basics,
    ex_box_pin_address_stability,
    ex_non_unpin_type,
    ex_pin_api_and_projection,
};

fn main() {
    ex_unpin_basics();
    ex_box_pin_address_stability();
    ex_non_unpin_type();
    ex_pin_api_and_projection();

    println!("\n== Extra notes ==");
    println!("Most types are Unpin; pinning primarily matters for `!Unpin` (self-referential, async state).");
}
