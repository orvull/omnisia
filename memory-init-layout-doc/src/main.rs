use memory_init_layout_doc::{
    ex_maybeuninit_array,
    ex_maybeuninit_out_param,
    ex_zeroing_note,
    ex_manuallydrop_basics,
    ex_manuallydrop_ffi_style,
    ex_niche_sizes,
    ex_nonzero_api,
};

fn main() {
    ex_maybeuninit_array();
    ex_maybeuninit_out_param();
    ex_zeroing_note();
    ex_manuallydrop_basics();
    ex_manuallydrop_ffi_style();
    ex_niche_sizes();
    ex_nonzero_api();
    println!("\n== Cheatsheet in comments below ==");
}
