# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-09-13

### Fixed
- Added README documentation to crate package metadata for crates.io.

## [0.1.0] - 2026-09-13

### Added
- Initial release of the Gleamler NIF framework.
- Core runtime library (`gleamler`) with ERTS C API wrappers and safe term abstractions.
- Procedural attribute macro `#[gleam_nif]` with `safe`, `dirty_cpu`, and `dirty_io` flags.
- Automatic Gleam code generator (`gleamler_codegen`) for `@external` bindings, custom types, and `gleam/dynamic/decode` decoders.
- Derive macros for `NifRecord`, `NifTuple`, `NifMap`, `NifUnitEnum`, and `NifTaggedEnum`.
- Native `process.Subject` and zero-allocation `SubjectSender` messaging for background OS threads.
- Process cancellation tokens via BEAM process monitors (`CancellationToken`).
- Cooperative scheduling with non-blocking continuations (`NifOutcome::Yield`).
- Streaming iterator bridge (`Yielder` -> `gleam/yielder.Yielder`).
- Task automation CLI with live reload support (`cargo xtask watch`).
- Comprehensive test suite including BEAM garbage collection leak verification and Valgrind suppressions.