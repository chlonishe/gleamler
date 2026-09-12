use std::env;
use std::fs;
use std::path::Path;

fn atomic_write(path: &std::path::Path, contents: impl AsRef<[u8]>) {
    let bytes = contents.as_ref();
    if let Ok(existing) = std::fs::read(path)
        && existing == bytes
    {
        return;
    }
    let tmp = path.with_extension("tmp");

    struct TmpGuard<'a>(&'a std::path::Path);
    impl<'a> Drop for TmpGuard<'a> {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(self.0);
        }
    }
    let _guard = TmpGuard(&tmp);

    std::fs::write(&tmp, bytes)
        .unwrap_or_else(|e| panic!("failed to write temp file {}: {}", tmp.display(), e));
    if cfg!(windows) {
        let _ = std::fs::remove_file(path);
    }
    std::fs::rename(&tmp, path).unwrap_or_else(|e| {
        panic!(
            "failed to rename {} → {}: {}",
            tmp.display(),
            path.display(),
            e
        )
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!(
            "Usage: {} <gleamler_crate_dir> <erl_out> <gleam_out> [--with-stress] [erl_module] [lib_name]",
            args[0]
        );
        std::process::exit(1);
    }

    let mut with_stress = false;
    let mut positional = Vec::new();
    for arg in &args[1..] {
        if arg == "--with-stress" {
            with_stress = true;
        } else {
            positional.push(arg.clone());
        }
    }

    if positional.len() < 3 {
        eprintln!("Error: missing required positional arguments");
        std::process::exit(1);
    }

    let crate_dir = &positional[0];
    let erl_out = &positional[1];
    let gleam_out = &positional[2];

    let nifs_rs = {
        let p_nifs = Path::new(crate_dir).join("src/nifs.rs");
        if p_nifs.exists() {
            p_nifs
        } else {
            Path::new(crate_dir).join("src/lib.rs")
        }
    };
    let stress_nifs_rs = Path::new(crate_dir).join("src/stress_nifs.rs");

    let nifs_source = fs::read_to_string(&nifs_rs)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", nifs_rs.display(), e));

    let nifs_functions = gleamler_codegen::parse_nif_functions(&nifs_source);
    let nifs_types = gleamler_codegen::parse_nif_types(&nifs_source);

    let (stress_functions, stress_types) = if with_stress && stress_nifs_rs.exists() {
        let stress_source = fs::read_to_string(&stress_nifs_rs)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", stress_nifs_rs.display(), e));
        (
            gleamler_codegen::parse_nif_functions(&stress_source),
            gleamler_codegen::parse_nif_types(&stress_source),
        )
    } else {
        (Vec::new(), Vec::new())
    };

    let mut functions = Vec::with_capacity(nifs_functions.len() + stress_functions.len());
    functions.extend(nifs_functions.clone());
    functions.extend(stress_functions.clone());

    let mut custom_types = Vec::with_capacity(nifs_types.len() + stress_types.len());
    custom_types.extend(nifs_types);
    custom_types.extend(stress_types);

    eprintln!(
        "Generated {} function(s) and {} custom type(s){}\n  → {}\n  → {}",
        functions.len(),
        custom_types.len(),
        if with_stress { " (with stress)" } else { "" },
        erl_out,
        gleam_out
    );

    let registered = gleamler_codegen::parse_init_nifs_list(&nifs_source);
    let warnings =
        gleamler_codegen::validate_nif_registry(&registered, &nifs_functions, &stress_functions);
    for w in warnings {
        eprintln!("warning: {w}");
    }

    const RESERVED_ERL_NAMES: &[&str] = &["init", "module_info", "record_info"];

    let mut erl_names = std::collections::BTreeSet::new();

    let erl_module = positional
        .get(3)
        .cloned()
        .or_else(|| gleamler_codegen::parse_init_nifs_module(&nifs_source))
        .unwrap_or_else(|| "gleamler_nif_ffi".to_string());
    let lib_name = positional
        .get(4)
        .cloned()
        .unwrap_or_else(|| "gleamler".to_string());

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

    let gleam_contents = gleamler_codegen::generate_gleam(&functions, &custom_types, &erl_module);

    atomic_write(std::path::Path::new(erl_out), erl_contents);
    atomic_write(std::path::Path::new(gleam_out), gleam_contents);
}
