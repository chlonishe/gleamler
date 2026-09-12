use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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
        let _ = fs::remove_file(path);
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

fn collect_rs_files(dir: &Path, with_stress: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rs_files(&path, with_stress));
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "stress_nifs.rs" && !with_stress {
                    continue;
                }
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!(
            "Usage: {} <crate_or_src_dir> <erl_out> <gleam_out> [--with-stress] [erl_module] [lib_name]",
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

    let input_dir = Path::new(&positional[0]);
    let erl_out = &positional[1];
    let gleam_out = &positional[2];

    let search_dir = if input_dir.join("src").exists() {
        input_dir.join("src")
    } else {
        input_dir.to_path_buf()
    };

    let rs_files = collect_rs_files(&search_dir, with_stress);
    if rs_files.is_empty() {
        eprintln!("warning: no .rs files found in {}", search_dir.display());
    }

    let mut all_functions = Vec::new();
    let mut all_custom_types = Vec::new();
    let mut registered_names = Vec::new();
    let mut discovered_module = None;

    for file_path in &rs_files {
        let Ok(source) = fs::read_to_string(file_path) else {
            continue;
        };

        let funcs = gleamler_codegen::parse_nif_functions(&source);
        let types = gleamler_codegen::parse_nif_types(&source);
        let reg = gleamler_codegen::parse_init_nifs_list(&source);

        if discovered_module.is_none() {
            discovered_module = gleamler_codegen::parse_init_nifs_module(&source);
        }

        all_functions.extend(funcs);
        all_custom_types.extend(types);
        registered_names.extend(reg);
    }

    let mut unique_types = Vec::new();
    let mut seen_types = std::collections::HashSet::new();
    for ct in all_custom_types {
        if seen_types.insert(ct.name.clone()) {
            unique_types.push(ct);
        }
    }

    let erl_module = positional
        .get(3)
        .cloned()
        .or(discovered_module)
        .unwrap_or_else(|| "gleamler_nif_ffi".to_string());

    let lib_name = positional
        .get(4)
        .cloned()
        .unwrap_or_else(|| "gleamler".to_string());

    const RESERVED_ERL_NAMES: &[&str] = &["init", "module_info", "record_info"];
    let mut erl_names = std::collections::BTreeSet::new();

    for func in &all_functions {
        let erl_name = func.alias.clone().unwrap_or_else(|| func.name.clone());

        if RESERVED_ERL_NAMES.contains(&erl_name.as_str()) {
            panic!(
                "gleamler_codegen: NIF name '{}' conflicts with a reserved Erlang function name in module '{}'",
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

    let warnings = gleamler_codegen::validate_nif_registry(&registered_names, &all_functions, &[]);
    for w in warnings {
        eprintln!("warning: {w}");
    }

    eprintln!(
        "Scanned {} file(s) in {}: generated {} function(s), {} type(s)\n  → {}\n  → {}",
        rs_files.len(),
        search_dir.display(),
        all_functions.len(),
        unique_types.len(),
        erl_out,
        gleam_out
    );

    let erl_contents = gleamler_codegen::generate_erl(&all_functions, &erl_module, &lib_name);
    let gleam_contents =
        gleamler_codegen::generate_gleam(&all_functions, &unique_types, &erl_module);

    atomic_write(std::path::Path::new(erl_out), erl_contents);
    atomic_write(std::path::Path::new(gleam_out), gleam_contents);
}
