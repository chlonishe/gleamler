# Gleamler

[Getting Started](#getting-started) | [Task Runner](#task-runner-cargo-xtask--cargo-x) | [Examples](examples/) | [Architecture](ARCHITECTURE.md)

Gleamler is a library for writing Erlang NIFs in safe Rust code with first-class support for Gleam. That means there should be no way to crash the BEAM (Erlang VM). The library provides facilities for generating the boilerplate for interacting with the BEAM, handles encoding and decoding of Erlang terms, generates matching Gleam bindings and decoders and catches Rust panics before they unwind into C.

While inspired by and building upon foundations established by Rustler, Gleamler is specifically designed for the Gleam type system and toolchain.

## Features

### Safety
The code you write in a Rust NIF should never be able to crash the BEAM. Panics are caught at the FFI boundary and converted into descriptive Erlang exceptions or Gleam `Result` types.

### Interop
Decoding and encoding Rust values into Erlang and Gleam terms is as easy as a function call. Common Rust standard library types (`Vec`, `Option`, `Result`, `HashMap`, `SystemTime`, `IpAddr`, primitives up to 128-bit) are supported out of the box.

### Code and Type Generation
Gleamler automatically scans your Rust NIF definitions and generates native Gleam `@external` bindings, corresponding custom types and `gleam/dynamic/decode` decoders for structs and enums annotated with `NifRecord`, `NifMap`, `NifTuple`, `NifUnitEnum`, or `NifTaggedEnum`.

### Resource Objects
Enables you to safely pass a reference to a Rust struct into Gleam code. The struct is reference-counted and will be automatically dropped by the BEAM garbage collector when it is no longer referenced.

### OTP and Process Integration
First-class support for Gleam's `process.Subject` enables background OS threads to stream messages directly to Gleam actors with zero allocation overhead in worker loops. Tasks can monitor process lifecycles via `CancellationToken` and exit automatically when the calling process terminates.

### Cooperative Scheduling
Long-running CPU tasks can cooperatively yield execution back to the BEAM scheduler using `NifOutcome::Yield`, or run on dirty scheduler pools via `dirty_cpu` and `dirty_io` flags.

---

## What it looks like

### Rust implementation

```rust
use gleamler::{gleam_nif, init_nifs, NifRecord};

#[derive(NifRecord, Debug)]
#[tag = "user"]
pub struct User {
    pub id: i64,
    pub name: String,
}

#[gleam_nif]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[gleam_nif]
fn get_user(id: i64, name: String) -> User {
    User { id, name }
}

init_nifs!();
```

### Calling from Gleam

Running `cargo x` automatically compiles the dynamic library, copies it to `priv/` and generates matching Gleam bindings and decoders:

```gleam
import gleam/io
import gleam/int
import my_nif

pub fn main() {
  let sum = my_nif.rust_add(10, 20)
  io.println("Sum: " <> int.to_string(sum))

  let user = my_nif.rust_get_user(1, "Alice")
  io.println("Hello, " <> user.name)
}
```

---

## Getting Started

### 1. Add a Rust Crate to your Gleam Project

Inside your Gleam project root, create a directory for your Rust NIF (for example, `native/my_nif`):

```bash
cargo new --lib native/my_nif
```

### 2. Configure `Cargo.toml`

Configure the crate as a dynamic library (`cdylib`) and add `gleamler` from crates.io in `native/my_nif/Cargo.toml`:

```toml
[package]
name = "my_nif"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
gleamler = "0.1.2"
```

### 3. Write your NIF

In `native/my_nif/src/lib.rs`:

```rust
use gleamler::{gleam_nif, init_nifs};

/// Adds two integers
#[gleam_nif]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

init_nifs!();
```

### 4. Build & Generate Gleam Stubs

Build the dynamic library in release mode:

```bash
cargo build --manifest-path native/my_nif/Cargo.toml --release
```

Copy the compiled library to your Gleam project's `priv/` directory:

- **Linux**: `cp target/release/libmy_nif.so priv/my_nif.so`
- **macOS**: `cp target/release/libmy_nif.dylib priv/my_nif.so`
- **Windows**: `copy target\release\my_nif.dll priv\my_nif.dll`

Generate matching Gleam bindings and decoders using `gleamler_codegen`:

```bash
cargo run --manifest-path native/my_nif/Cargo.toml -p gleamler_codegen -- \
    native/my_nif \
    src/my_nif_ffi.erl \
    src/my_nif.gleam
```

### 5. Call from Gleam

Now simply call the generated function from your Gleam code:

```gleam
import gleam/io
import gleam/int
import my_nif

pub fn main() {
  let result = my_nif.rust_add(15, 27)
  io.println("Result: " <> int.to_string(result))
}
```

---

## Task Runner (`cargo xtask` / `cargo x`)

Gleamler includes an integrated automation task runner. Using `.cargo/config.toml`, commands can be invoked via `cargo xtask` or the shorthand `cargo x`:

| Command | Short Alias | Description |
|---|---|---|
| `cargo x` | `cargo x b` | Default command. Builds NIF, installs to `priv/` and runs codegen |
| `cargo x build --release` | | Compiles optimized release build |
| `cargo x watch` | `cargo x w` | Watches source files, rebuilds and runs Gleam tests |
| `cargo x test` | `cargo x t` | Runs Rust unit tests, Gleam test suite and GC leak test |
| `cargo x test --leak` | | Runs 100-iteration BEAM garbage collector leak verification |
| `cargo x test --valgrind` | | Runs Valgrind Memcheck with ERTS thread suppressions |
| `cargo x codegen` | `cargo x gen` | Regenerates Gleam stubs and decoders without full recompile |
| `cargo x example <name>` | `cargo x ex` | Builds a standalone example from `examples/` |
| `cargo x new <name>` | | Scaffolds a new NIF crate template |
| `cargo x clean` | `cargo x c` | Cleans `priv/`, `build/` and cargo `target/` directories |
| `cargo x ci` | | Runs full local CI checks (fmt, clippy, tests, examples) |

---

## Supported NIF and OTP Versions

The minimum supported NIF version for a library should be defined via Cargo features:

| NIF Version | Minimum Erlang/OTP Version | Cargo Feature |
|---|---|---|
| 2.14 | Erlang/OTP 21 | `nif_version_2_14` |
| 2.15 | Erlang/OTP 22 | `nif_version_2_15` |
| 2.16 | Erlang/OTP 24 | `nif_version_2_16` |
| 2.17 (default) | Erlang/OTP 26 | `nif_version_2_17` |
| 2.18 | Erlang/OTP 27 | `nif_version_2_18` |

---

## Minimal Supported Rust Version (MSRV)

Gleamler requires Rust edition 2024 (Rust 1.85 or newer).

---

## Acknowledgements

Gleamler builds upon concepts, internal FFI abstractions and safety patterns pioneered by the [Rustler](https://github.com/rusterlium/rustler) project and its contributors:

- [hansihe](https://github.com/hansihe)
- [The Contributors of the Rustler Project](https://github.com/rusterlium/rustler/graphs/contributors)

We are grateful for their work in bringing safe Rust integration to the BEAM ecosystem.

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
