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
    let erl_module = args
        .get(4)
        .cloned()
        .unwrap_or_else(|| "gleamler_nif_ffi".to_string());
    let lib_name = args
        .get(5)
        .cloned()
        .unwrap_or_else(|| "gleamler".to_string());

    let source = fs::read_to_string(src_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", src_path, e));

    let functions = gleamler_codegen::parse_nif_functions(&source);
    eprintln!(
        "Generated {} function(s)\n  → {}\n  → {}",
        functions.len(),
        erl_out,
        gleam_out
    );

    let registered = gleamler_codegen::parse_init_nifs_list(&source);
    if !registered.is_empty() {
        for f in &functions {
            if !registered.contains(&f.name) {
                eprintln!(
                    "warning: #[gleam_nif] fn `{}` is not listed in init_nifs! — \
                     its stubs will exit(nif_library_not_loaded) at runtime",
                    f.name
                );
            }
        }
        let declared: std::collections::BTreeSet<_> =
            functions.iter().map(|f| f.name.as_str()).collect();
        for name in &registered {
            if !declared.contains(name.as_str()) {
                eprintln!(
                    "warning: `{name}` is listed in init_nifs! but has no #[gleam_nif] function"
                );
            }
        }
    }

    const RESERVED_ERL_NAMES: &[&str] = &["init", "module_info", "record_info"];

    let mut erl_names = std::collections::BTreeSet::new();

    for func in &functions {
        let erl_name = func.alias.clone().unwrap_or_else(|| func.name.clone());

        if RESERVED_ERL_NAMES.contains(&erl_name.as_str()) {
            panic!(
                "gleamler_codegen: NIF name '{}' conflicts with a reserved Erlang function name \
                 in module '{}' (names like init, module_info cannot be used as NIF aliases)",
                erl_name, erl_module
            );
        }

        if !erl_names.insert(erl_name.clone()) {
            panic!(
                "gleamler_codegen: duplicate exported NIF name '{}' (alias collision) in file {:?}",
                erl_name, source
            );
        }
    }

    let erl_contents = gleamler_codegen::generate_erl(&functions, &erl_module, &lib_name);
    let gleam_contents = gleamler_codegen::generate_gleam(&functions, &erl_module);

    fs::write(erl_out, erl_contents)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", erl_out, e));
    fs::write(gleam_out, gleam_contents)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", gleam_out, e));
}
