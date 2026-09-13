# Contributing to Gleamler

Thank you for your interest in contributing to Gleamler. This document outlines guidelines for reporting bugs, submitting features, and working with the codebase.

---

## Development Setup

### Prerequisites

- Rust (edition 2024, Rust 1.85 or newer)
- Erlang/OTP (version 26 or newer)
- Gleam (version 1.0 or newer)

### Building from Source

Clone the repository and run the default build task:

```bash
git clone https://github.com/chlonishe/gleamler.git
cd gleamler
cargo x
```

### Live Watch Mode

To develop with live recompilation and automatic test runs on save:

```bash
cargo x w
```

---

## Testing

Gleamler contains multiple test layers across Rust, Gleam, and the BEAM runtime:

- Run all test suites: `cargo x t`
- Run the BEAM garbage collector leak verification: `cargo x test --leak`
- Run local CI checks: `cargo x ci`

All pull requests must pass `cargo x ci` before being merged.

---

## Code Standards

- Rust code must be formatted with `rustfmt` (`cargo fmt --all`).
- Gleam code must be formatted with `gleam format`.
- Clippy must produce zero warnings:
  ```bash
  cargo clippy --workspace --all-targets -- -W warnings
  ```

---

## Pull Request Process

1. Fork the repository and create your branch from `master`.
2. If you add new functionality, include corresponding tests in `test/gleamler_test.gleam` or Rust unit tests.
3. If you modify NIF macro generation or codegen logic, ensure that generated Gleam and Erlang files remain valid and cleanly formatted.
4. Document notable changes in `CHANGELOG.md` under an `[Unreleased]` section.
5. Verify that `cargo x ci` passes cleanly on your machine before pushing.

---

## License

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed under the MIT and Apache 2.0 licenses, without any additional terms or conditions.