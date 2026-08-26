use std::env;
use std::fs;
use std::path::Path;

fn write_if_changed(path: &str, contents: impl AsRef<[u8]>) {
    let bytes = contents.as_ref();
    if let Ok(existing) = fs::read(path) {
        if existing == bytes {
            return;
        }
    }
    fs::write(path, bytes)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", path, e));
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!(
            "Usage: {} <gleamler_crate_dir> <erl_out> <gleam_out> [erl_module] [lib_name]",
            args[0]
        );
        std::process::exit(1);
    }

    let crate_dir = &args[1];
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

    let nifs_rs = Path::new(crate_dir).join("src/nifs.rs");
    let stress_nifs_rs = Path::new(crate_dir).join("src/stress_nifs.rs");

    let mut functions = Vec::new();

    let nifs_source = fs::read_to_string(&nifs_rs)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", nifs_rs.display(), e));
    functions.extend(gleamler_codegen::parse_nif_functions(&nifs_source));

    // stress_nifs.rs is optional — absence means empty list (e.g. cross-compile)
    if stress_nifs_rs.exists() {
        let stress_source = fs::read_to_string(&stress_nifs_rs)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", stress_nifs_rs.display(), e));
        functions.extend(gleamler_codegen::parse_nif_functions(&stress_source));
    }

    eprintln!(
        "Generated {} function(s)\n  → {}\n  → {}",
        functions.len(),
        erl_out,
        gleam_out
    );

    let registered = gleamler_codegen::parse_init_nifs_list(&nifs_source);
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
                "gleamler_codegen: duplicate exported NIF name '{}' (alias collision)",
                erl_name
            );
        }
    }

    let erl_contents = gleamler_codegen::generate_erl(&functions, &erl_module, &lib_name);
    let gleam_contents = gleamler_codegen::generate_gleam(&functions, &erl_module);

    write_if_changed(erl_out, erl_contents);
    write_if_changed(gleam_out, gleam_contents);
}
