use box_doc::{
    example_basic,
    example_recursive,
    example_trait_objects,
    example_borrow,
};

fn main() {
    println!("--- Example 1: Basic ---");
    example_basic();

    println!("\n--- Example 2: Recursive ---");
    example_recursive();

    println!("\n--- Example 3: Trait objects ---");
    example_trait_objects();

    println!("\n--- Example 4: Borrow ---");
    example_borrow();
}
