use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

#[derive(Parser, Debug)]
#[command(name = "xtask", about = "Gleamler build & test automation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Build {
        #[arg(long)]
        release: bool,
        #[arg(long)]
        stress: bool,
        #[arg(long)]
        target: Option<String>,
    },

    Example {
        name: String,
        #[arg(long)]
        release: bool,
    },

    New {
        name: String,
    },

    Codegen {
        #[arg(long)]
        with_stress: bool,
    },

    Test {
        #[arg(long, group = "test_group")]
        rust: bool,
        #[arg(long, group = "test_group")]
        gleam: bool,
        #[arg(long, group = "test_group")]
        leak: bool,
        #[arg(long, group = "test_group")]
        valgrind: bool,

        #[arg(long, group = "test_group")]
        all: bool,
    },

    Clean,

    Ci {
        #[arg(long)]
        fast: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    let root = workspace_root()?;
    sh.change_dir(&root);

    match cli.command {
        Commands::Build {
            release,
            stress,
            target,
        } => {
            build(&sh, "gleamler", release, stress, target.as_deref())?;
        }
        Commands::Example { name, release } => {
            build_example(&sh, &name, release)?;
        }
        Commands::New { name } => {
            scaffold_new_nif(&name)?;
        }
        Commands::Codegen { with_stress } => {
            codegen(&sh, "gleamler", with_stress)?;
        }
        Commands::Test {
            rust,
            gleam,
            leak,
            valgrind,
            all,
        } => {
            let run_all = all || (!rust && !gleam && !leak && !valgrind);
            if run_all || rust {
                test_rust(&sh)?;
            }
            if run_all || gleam {
                test_gleam(&sh)?;
            }
            if run_all || leak {
                test_leak(&sh)?;
            }
            if valgrind {
                test_valgrind(&sh)?;
            }
        }
        Commands::Clean => {
            clean(&sh)?;
        }
        Commands::Ci { fast } => {
            ci(&sh, fast)?;
        }
    }

    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let manifest = env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?;
    PathBuf::from(manifest)
        .parent()
        .context("xtask must reside in workspace root")
        .map(|p| p.to_path_buf())
}

fn build(
    sh: &Shell,
    package: &str,
    release: bool,
    stress: bool,
    target: Option<&str>,
) -> Result<()> {
    println!("==> Building Rust NIF ({package})...");
    let mut args = vec!["build", "-p", package];
    if release {
        args.push("--release");
    }
    if stress {
        args.push("--features");
        args.push("stress");
    }
    if let Some(t) = target {
        args.push("--target");
        args.push(t);
    }
    cmd!(sh, "cargo {args...}").run()?;

    let (src_name, dst_name) = artifact_names(target, package);
    let target_dir = resolve_target_dir(target);
    let profile = if release { "release" } else { "debug" };
    let src = target_dir.join(profile).join(&src_name);

    if !src.exists() {
        bail!("Build artifact not found: {}", src.display());
    }

    println!("==> Installing NIF -> priv/{dst_name}");
    sh.create_dir("priv")?;
    sh.copy_file(&src, format!("priv/{dst_name}"))?;

    codegen(sh, package, stress)?;
    println!("==> Building Gleam...");
    cmd!(sh, "gleam build").run()?;

    println!("==> Build complete for {package}!");
    Ok(())
}

fn build_example(sh: &Shell, name: &str, release: bool) -> Result<()> {
    let example_path = Path::new("examples").join(name);
    if !example_path.exists() {
        bail!("Example '{name}' not found in examples/ directory!");
    }

    println!("==> Activating example '{name}'...");
    build(sh, name, release, false, None)?;
    println!("\nExample '{name}' is ready! You can now run:\n  gleam run");
    Ok(())
}

fn codegen(sh: &Shell, crate_dir: &str, with_stress: bool) -> Result<()> {
    println!("==> Generating FFI stubs for {crate_dir}...");
    let mut args = vec![
        "run",
        "-p",
        "gleamler_codegen",
        "--",
        crate_dir,
        "src/gleamler_nif_ffi.erl",
        "src/gleamler_nif.gleam",
    ];
    if with_stress {
        args.push("--with-stress");
    }
    cmd!(sh, "cargo {args...}").run()?;

    let _ = cmd!(sh, "gleam format src/gleamler_nif.gleam").run();
    Ok(())
}

fn scaffold_new_nif(name: &str) -> Result<()> {
    let path = PathBuf::from(name);
    let crate_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my_nif");

    if path.exists() {
        bail!("Directory '{}' already exists!", path.display());
    }

    fs::create_dir_all(path.join("src"))?;

    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"

[lib]
crate-type = ["cdylib"]
test = false

[dependencies]
gleamler = {{ path = "../../gleamler" }}
"#
    );

    let lib_rs = r#"use gleamler::{gleam_nif, init_nifs};

/// Adds two integers
#[gleam_nif]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// Greets by name
#[gleam_nif]
fn greet(name: String) -> String {
    format!("Hello from {}!", name)
}

init_nifs!();
"#;

    fs::write(path.join("Cargo.toml"), cargo_toml)?;
    fs::write(path.join("src/lib.rs"), lib_rs)?;

    println!("==> Created new Gleamler NIF crate at: {}", path.display());
    println!("To build it, add it to your workspace Cargo.toml and run:");
    println!("  cargo xtask example {}", crate_name);
    Ok(())
}

fn test_rust(sh: &Shell) -> Result<()> {
    println!("==> Testing all Rust workspace crates...");
    cmd!(sh, "cargo test --workspace").run()?;

    println!("==> Testing gleamler with --features stress...");
    cmd!(sh, "cargo test -p gleamler --features stress").run()?;
    Ok(())
}

fn test_gleam(sh: &Shell) -> Result<()> {
    if !Path::new("priv").exists() {
        println!("priv/ missing, building first...");
        build(sh, "gleamler", true, true, None)?;
    }
    println!("==> Running Gleam tests...");
    cmd!(sh, "gleam test").run()?;
    Ok(())
}

fn test_leak(sh: &Shell) -> Result<()> {
    if !Path::new("priv").exists() {
        println!("priv/ missing, building first...");
        build(sh, "gleamler", true, true, None)?;
    }
    println!("==> Running resource leak test...");
    let ebin = "build/dev/erlang/gleamler/ebin";
    sh.create_dir(ebin)?;
    cmd!(sh, "erlc -o {ebin} src/resource_leak_test.erl").run()?;
    cmd!(
        sh,
        "erl +S 1:1 -noshell -pa {ebin} -s resource_leak_test run -s init stop"
    )
    .run()?;
    println!("==> Leak test passed");
    Ok(())
}

fn test_valgrind(sh: &Shell) -> Result<()> {
    if !Path::new("priv/gleamler.so").exists() {
        println!("NIF not found, building first...");
        build(sh, "gleamler", true, true, None)?;
    }
    println!("==> Running Valgrind...");

    let find_out = cmd!(sh, "find build/dev/erlang -name ebin").read()?;
    let pa_args: Vec<String> = find_out
        .lines()
        .filter(|l| !l.is_empty())
        .flat_map(|l| vec!["-pa".to_string(), l.trim().to_string()])
        .collect();

    if pa_args.is_empty() {
        bail!("No ebin dirs found. Run `gleam build` first.");
    }

    cmd!(sh, "valgrind --leak-check=full --show-leak-kinds=definite,indirect --errors-for-leak-kinds=definite,indirect --error-exitcode=1 --suppressions=valgrind.supp erl +S 1:1 -noshell {pa_args...} -s resource_leak_test run -s init stop").run()?;
    println!("==> Valgrind passed");
    Ok(())
}

fn clean(sh: &Shell) -> Result<()> {
    println!("==> Cleaning...");
    let _ = sh.remove_path("priv");
    let _ = sh.remove_path("build");
    cmd!(sh, "cargo clean").run()?;
    Ok(())
}

fn has_cargo_subcommand(sh: &Shell, cmd: &str) -> bool {
    cmd!(sh, "cargo {cmd} --version")
        .ignore_stderr()
        .read()
        .is_ok()
}

fn ci(sh: &Shell, fast: bool) -> Result<()> {
    println!("==> CI: cargo fmt check");
    cmd!(sh, "cargo fmt --all -- --check").run()?;

    println!("==> CI: clippy");
    cmd!(sh, "cargo clippy --workspace --all-targets -- -W warnings").run()?;

    println!("==> CI: rust tests");
    test_rust(sh)?;

    if !fast {
        if has_cargo_subcommand(sh, "hack") {
            println!("==> CI: feature powerset");
            cmd!(sh, "cargo hack check -p gleamler --feature-powerset --at-least-one-of nif_version_2_14,nif_version_2_15,nif_version_2_16,nif_version_2_17,nif_version_2_18 --lib").run()?;
        } else {
            println!("==> CI: `cargo-hack` not installed, skipping feature powerset");
        }
    }

    println!("==> CI: gleam build + format + test");
    build(sh, "gleamler", true, true, None)?;
    cmd!(sh, "gleam format --check").run()?;
    test_gleam(sh)?;

    if !fast {
        if has_cargo_subcommand(sh, "deny") {
            println!("==> CI: cargo deny");
            cmd!(sh, "cargo deny check all").run()?;
        } else {
            println!("==> CI: `cargo-deny` not installed, skipping cargo deny");
        }
    }

    println!("==> CI dry-run complete!");
    Ok(())
}

fn artifact_names(target: Option<&str>, package: &str) -> (String, String) {
    let os = match target {
        Some(t) if t.contains("windows") => "windows",
        Some(t) if t.contains("darwin") || t.contains("apple") => "macos",
        Some(_) => "linux",
        None => env::consts::OS,
    };

    match os {
        "windows" => (format!("{package}.dll"), "gleamler.dll".into()),
        "macos" => (format!("lib{package}.dylib"), "gleamler.so".into()),
        _ => (format!("lib{package}.so"), "gleamler.so".into()),
    }
}

fn resolve_target_dir(target: Option<&str>) -> PathBuf {
    let mut p = PathBuf::from("target");
    if let Some(t) = target {
        p.push(t);
    } else if let Ok(t) = env::var("CARGO_BUILD_TARGET") {
        p.push(t);
    }
    p
}
