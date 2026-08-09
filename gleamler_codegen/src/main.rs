use std::fs;
use std::path::Path;
use std::collections::BTreeMap;
use gleamler_codegen::{parse_nif_functions, generate_erl, generate_gleam};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let source = args
        .get(1)
        .map(|s| Path::new(s).to_path_buf())
        .unwrap_or_else(|| Path::new("gleamler/src").to_path_buf());
    let erl_out = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "src/gleamler_nif_ffi.erl".to_string());
    let gleam_out = args
        .get(3)
        .cloned()
        .unwrap_or_else(|| "src/gleamler_nif.gleam".to_string());

    let scan_dir = if source.is_file() {
        source.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| Path::new(".").to_path_buf())
    } else {
        source
    };

    let mut functions = BTreeMap::new();

    visit_dirs(&scan_dir, &mut |path| {
        if let Ok(content) = fs::read_to_string(path) {
            for func in parse_nif_functions(&content) {
                functions.insert(func.name.clone(), func);
            }
        }
    })
    .expect("failed to read source directory");

    let functions: Vec<_> = functions.into_values().collect();

    fs::write(&erl_out, generate_erl(&functions))
        .expect("failed to write erl output");
    fs::write(&gleam_out, generate_gleam(&functions))
        .expect("failed to write gleam output");

    println!("Generated {} function(s)", functions.len());
    println!("  → {}", erl_out);
    println!("  → {}", gleam_out);
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                cb(&path);
            }
        }
    }
    Ok(())
}