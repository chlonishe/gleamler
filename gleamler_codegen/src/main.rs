use std::fs;
use gleamler_codegen::{parse_nif_functions, generate_erl, generate_gleam};

fn main() {
    let lib_rs = fs::read_to_string("gleamler/src/lib.rs")
        .expect("gleamler/src/lib.rs not found — run this from the project root");

    let functions = parse_nif_functions(&lib_rs);

    fs::write("src/gleamler_nif_ffi.erl", generate_erl(&functions))
        .expect("failed to write src/gleamler_nif_ffi.erl");
    fs::write("src/gleamler_nif.gleam", generate_gleam(&functions))
        .expect("failed to write src/gleamler_nif.gleam");

    println!("Generated {} function(s)", functions.len());
    println!("  → src/gleamler_nif_ffi.erl");
    println!("  → src/gleamler_nif.gleam");
}