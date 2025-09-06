use std::collections::HashMap;
use rust_desclarative_macros_doc::{
    mprintln, show_kind, make_vec, make_map, over, count_args, tiny_vec, hashmap, cmds, my_debug,
};

fn main() {
    mprintln!("== 1) basics");
    mprintln!("hello {}", "macros");

    mprintln!("\n== 2) fragment specifiers");
    show_kind!(foo);                     // ident
    show_kind!(std::collections::HashMap::<i32, i32>); // path
    show_kind!(Option<Result<i32, ()>>); // ty
    show_kind!(1 + 2 * 3);               // expr
    show_kind!({ let z = 1; z + 1 });    // block
    show_kind!(cfg(feature = "x"));      // meta
    show_kind!(<T as Into<U>>::into);    // tt (generic path)

    mprintln!("\n== 3) repetitions / separators / trailing comma");
    let a = make_vec![10, 20, 30,];
    mprintln!("make_vec -> {:?}", a);
    let m = make_map!{
        "a" => 1,
        "b" => 2,
    };
    mprintln!("make_map -> {:?}", m);

    mprintln!("\n== 4) overloading by pattern");
    over!(123);
    over!(10, 20);
    over!(answer = 42);

    mprintln!("\n== 5) count args");
    mprintln!("count() 0 => {}", count_args![]);
    mprintln!("count() 3 => {}", count_args![a, b, c]);
    mprintln!("count() 5 => {}", count_args![1, (x, y), {3}, foo, bar]);

    mprintln!("\n== 6) container builders");
    let v1 = tiny_vec![1, 2, 3];
    let v2 = tiny_vec![9; 4];
    mprintln!("tiny_vec lits     -> {:?}", v1);
    mprintln!("tiny_vec repeat   -> {:?}", v2);
    let hm: HashMap<&'static str, i32> = hashmap!{ "x" => 1, "y" => 2 };
    mprintln!("hashmap -> {:?}", hm);

    mprintln!("\n== 7) tt-muncher DSL");
    let result = cmds! { add 3; add 4; sub 1; add (2*2); };
    mprintln!("cmds! result = {}", result);

    mprintln!("\n== 8) hygiene & $crate");
    let __val = 999;
    let x = 123;
    let got = my_debug!(x * 2);
    mprintln!("my_debug returned {}", got);

    let _ok = make_vec![ "a", "b", "c", ];
}
