use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "xtask", about = "Gleamler build & test automation", default_subcommand = "build")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build Rust NIF, install to priv/, and generate Gleam stubs
    #[command(alias = "b")]
    Build {
        #[arg(long)]
        release: bool,
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        stress: bool,
        #[arg(long)]
        target: Option<String>,
    },

    /// Build a standalone example from examples/
    #[command(alias = "ex")]
    Example {
        name: String,
        #[arg(long)]
        release: bool,
    },

    /// Scaffold a new NIF crate template
    New {
        name: String,
    },

    /// Regenerate Gleam FFI and decoder stubs
    #[command(alias = "gen")]
    Codegen {
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        with_stress: bool,
    },

    /// Run test suites (Rust, Gleam, leak, valgrind)
    #[command(alias = "t")]
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

    /// Clean priv/, build/, and cargo target/
    #[command(alias = "c")]
    Clean,

    /// Run full CI checks (fmt, clippy, tests, examples)
    Ci {
        #[arg(long)]
        fast: bool,
    },

    /// Watch for changes, recompile NIF, run codegen and tests
    #[command(alias = "w")]
    Watch {
        #[arg(long, default_value = "gleamler")]
        package: String,
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        stress: bool,
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
        Commands::Watch { package, stress } => {
            watch(&sh, &package, stress)?;
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
    if package == "gleamler" {
        args.push("--features");
        args.push(if stress { "nifs,stress" } else { "nifs" });
    } else if stress {
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
    let example_dir = format!("examples/{name}");
    if !Path::new(&example_dir).exists() {
        bail!("Example '{name}' not found in examples/ directory!");
    }

    println!("==> Building Rust example '{name}'...");
    let mut args = vec!["build", "-p", name];
    if release {
        args.push("--release");
    }
    cmd!(sh, "cargo {args...}").run()?;

    let (src_name, dst_name) = artifact_names(None, name);
    let target_dir = resolve_target_dir(None);
    let profile = if release { "release" } else { "debug" };
    let src = target_dir.join(profile).join(&src_name);

    println!("==> Installing NIF -> priv/{dst_name}");
    sh.create_dir("priv")?;
    sh.copy_file(&src, format!("priv/{dst_name}"))?;

    println!("==> Generating standalone FFI for {name}...");
    let erl_out = format!("{example_dir}/{name}_ffi.erl");
    let gleam_out = format!("{example_dir}/{name}.gleam");
    let erl_module = "gleamler_nif_ffi";
    cmd!(sh, "cargo run -p gleamler_codegen -- {example_dir} {erl_out} {gleam_out} {erl_module} {name}").run()?;
    let _ = cmd!(sh, "gleam format {gleam_out}").run();

    println!("\n[OK] Example '{name}' is ready!");
    println!("Generated standalone files:");
    println!("  -> {erl_out}");
    println!("  -> {gleam_out}");
    Ok(())
}

fn codegen(sh: &Shell, crate_dir: &str, with_stress: bool) -> Result<()> {
    let resolved_dir = if Path::new(crate_dir).exists() {
        crate_dir.to_string()
    } else {
        format!("examples/{crate_dir}")
    };

    println!("==> Generating FFI stubs for {resolved_dir}...");
    let mut args = vec![
        "run",
        "-p",
        "gleamler_codegen",
        "--",
        &resolved_dir,
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
    println!("==> Ensuring main NIF is built...");
    build(sh, "gleamler", true, true, None)?;

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

    println!("==> CI: checking examples...");
    for example in ["hello", "counter", "dirty", "serde_demo", "async_worker"] {
        build_example(sh, example, false)?;
    }

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
        "windows" => (format!("{package}.dll"), format!("{package}.dll")),
        "macos" => (format!("lib{package}.dylib"), format!("{package}.so")),
        _ => (format!("lib{package}.so"), format!("{package}.so")),
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

fn watch(sh: &Shell, package: &str, stress: bool) -> Result<()> {
    println!("==> Starting Gleamler Watch Mode for `{package}`...");
    println!("==> Watching for changes in `.rs` and `.gleam` files (ignoring build/, target/, priv/)...");

    run_watch_cycle(sh, package, stress);

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(Path::new("gleamler"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new("src"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new("test"), RecursiveMode::Recursive)?;
    if Path::new("examples").exists() {
        watcher.watch(Path::new("examples"), RecursiveMode::Recursive)?;
    }

    let debounce_duration = Duration::from_millis(300);
    let mut last_run = Instant::now();

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if !matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                ) {
                    continue;
                }

                let should_trigger = event.paths.iter().any(|path| {
                    let s = path.to_string_lossy();
                    if s.contains("target") || s.contains("build") || s.contains("priv") || s.contains(".git") {
                        return false;
                    }
                    s.ends_with(".rs") || s.ends_with(".gleam")
                });

                if should_trigger && last_run.elapsed() >= debounce_duration {
                    while rx.try_recv().is_ok() {}

                    println!("\n--------------------------------------------------");
                    println!("==> File change detected! Rebuilding...");
                    run_watch_cycle(sh, package, stress);
                    last_run = Instant::now();
                }
            }
            Ok(Err(e)) => eprintln!("Watch error: {e}"),
            Err(_) => break,
        }
    }

    Ok(())
}

fn run_watch_cycle(sh: &Shell, package: &str, stress: bool) {
    let start = Instant::now();

    if let Err(e) = build(sh, package, false, stress, None) {
        eprintln!("[FAIL] NIF Build / Codegen error:\n{e}");
        return;
    }

    println!("==> Running `gleam test`...");
    match cmd!(sh, "gleam test").run() {
        Ok(_) => {
            let elapsed = start.elapsed().as_millis();
            println!("\x1b[32m[SUCCESS] All tests passed! ({elapsed}ms)\x1b[0m");
        }
        Err(e) => {
            eprintln!("\x1b[31m[FAIL] Gleam tests failed:\x1b[0m\n{e}");
        }
    }
}