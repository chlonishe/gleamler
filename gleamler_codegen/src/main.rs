use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!(
            "Usage: {} <source.rs> <erl_out> <gleam_out> [erl_module] [lib_name]",
            args[0]
        );
        std::process::exit(1);
    }

    let src_path = &args[1];
    let erl_out = &args[2];
    let gleam_out = &args[3];
    let erl_module = args.get(4).cloned().unwrap_or_else(|| "gleamler_nif_ffi".to_string());
    let lib_name = args.get(5).cloned().unwrap_or_else(|| "gleamler".to_string());

    let source = fs::read_to_string(src_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", src_path, e));

    let functions = gleamler_codegen::parse_nif_functions(&source);
    eprintln!(
        "Generated {} function(s)\n  → {}\n  → {}",
        functions.len(),
        erl_out,
        gleam_out
    );

    let mut erl_names = std::collections::BTreeSet::new();
    
    for func in &functions {
        let erl_name = func.alias.clone().unwrap_or_else(|| func.name.clone());
        if !erl_names.insert(erl_name.clone()) {
            panic!(
                "gleamler_codegen: duplicate exported NIF name '{}' (alias collision) in file {:?}",
                erl_name, source
            );
        }
    }
    
    let erl_contents = gleamler_codegen::generate_erl(&functions, &erl_module, &lib_name);
    let gleam_contents = gleamler_codegen::generate_gleam(&functions, &erl_module);

    fs::write(erl_out, erl_contents).unwrap_or_else(|e| panic!("failed to write {}: {}", erl_out, e));
    fs::write(gleam_out, gleam_contents).unwrap_or_else(|e| panic!("failed to write {}: {}", gleam_out, e));
}