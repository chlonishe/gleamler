# Gleamler Architecture and Internals

This document describes the architectural design, memory safety guarantees, scheduling model and type mapping mechanisms of **Gleamler**.

---

## Architecture Overview

Gleamler is organized as a Cargo workspace with distinct responsibilities:

```text
[Rust Source (.rs)]
     │
     ├── syn AST Parsing ─────────► [gleamler_codegen]
     │                                    │
     │                                    ├──► [Erlang FFI (.erl)]
     │                                    └──► [Gleam Bindings (.gleam)]
     │
     └── cargo build (cdylib) ───► [lib<nif>.so / .dll]
                                          │
                                          ▼
                                    [priv/ directory]
                                          ▲
[Gleam Runtime] ── erlang:load_nif ───────┘
```

- **`gleamler`**: The core runtime library. Provides raw C ABI declarations (`sys`), safe wrappers for Erlang terms and environments, memory allocators, resource abstractions and native type implementations.
- **`gleamler_macros`**: Procedural macros (`#[gleam_nif]`, `init_nifs!`, `#[derive(NifRecord)]`, etc.). Generates C exports, arity assertions, panic barriers and encoding/decoding templates.
- **`gleamler_codegen`**: An AST-based code generator. Scans Rust source files using `syn` and outputs matching Gleam type definitions, `gleam/dynamic/decode` decoders and Erlang load stubs without requiring a compiled binary.
- **`xtask`**: Project automation CLI supporting build, watch mode, memory leak verification and CI pipelines.

---

## Type System and ABI Mapping

Gleam compiles to Erlang source files and runs on the BEAM virtual machine. Gleamler maps Rust types directly to the runtime representations used by Gleam:

| Rust Type | Erlang / BEAM Representation | Generated Gleam Type |
|---|---|---|
| `i8` .. `i128`, `u8` .. `u128`, `isize`, `usize` | `integer()` (small int or bignum) | `Int` |
| `f32`, `f64` | `float()` | `Float` |
| `bool` | `true \| false` (atoms) | `Bool` |
| `String`, `&str` | `binary()` (UTF-8) | `String` |
| `Binary<'a>`, `&[u8]`, `BitArray` | `binary()` | `BitArray` |
| `Option<T>` | `{some, Term} \| none` | `option.Option(T)` |
| `ErlOption<T>` | `Term \| undefined` | `option.Option(T)` |
| `Result<T, E>` | `{ok, T} \| {error, E}` | `Result(T, E)` |
| `Vec<T>`, `&[T]` | `[Term]` (linked list) | `List(T)` |
| `HashMap<K, V>`, `BTreeMap<K, V>` | `map()` | `dict.Dict(K, V)` |
| `(A, B, ...)` (up to 12 elements) | `{A, B, ...}` (tuple) | `#(A, B, ...)` |
| `Subject<'a, T>` | `{subject, Pid, Ref}` | `process.Subject(T)` |
| `ResourceArc<T>` | `enif_make_resource` pointer | `Resource(T)` (opaque) |
| `#[derive(NifRecord)]` | `{tag, Field1, Field2, ...}` | Record Custom Type |
| `#[derive(NifUnitEnum)]` | `atom()` (snake_case) | Enum Custom Type |
| `#[derive(NifTaggedEnum)]` | `atom() \| {variant, Field1, ...}` | Custom Type with Constructors |
| `#[derive(NifMap)]` | `#{<<"key">> => Value}` | `dynamic.Dynamic` + Decoder |

### Tag Validation and Safe Decoding

For `NifRecord` and `NifTaggedEnum`, Gleamler enforces strict tag verification during decoding:
- An internal helper (`check_tag`) verifies that index `0` of the tuple matches the expected atom before reading remaining fields.
- For `NifRecord`, decoding supports both Erlang tuple records (1-based index) and serialized maps/JSON (string keys) through `decode.one_of`.

---

## Memory Management

### 1. ERTS Allocator Integration (`alloc.rs`)

When the `allocator` feature is active, Gleamler configures `EnifAllocator` as the global Rust allocator (`#[global_allocator]`):
- Standard heap allocations are forwarded to `enif_alloc`, `enif_free` and `enif_realloc`, making NIF memory visible to BEAM system monitors.
- On 64-bit architectures, ERTS guarantees 16-byte alignment (`ENIF_MAX_ALIGN`).
- For types requiring higher alignment (such as 32-byte or 64-byte AVX/SIMD buffers), `EnifAllocator` transparently falls back to `std::alloc::System` to prevent misaligned pointer faults.

### 2. Resources (`ResourceArc<T>`)

A resource is a heap-allocated Rust structure managed by BEAM reference counting:
- **Alignment:** When allocating memory via `enif_alloc_resource`, `align_alloced_mem_for_struct` calculates the necessary offset to satisfy `std::mem::align_of::<T>()`.
- **Destruction:** When BEAM garbage collection determines all references have been dropped, `resource_destructor` takes ownership of the memory, calls `std::ptr::read::<T>()` to trigger `Drop` and optionally invokes a custom `destructor(env)` callback.
- **Registration:** Standard library resources (`CancellationResource`, `Yielder`) are registered automatically during initialization. User resources can be auto-registered using `#[resource_impl]` or explicitly in `on_load`.

### 3. Environment Lifetimes (`Env<'a>`)

To prevent Erlang terms from outliving the NIF call (which causes VM memory corruption), `Env<'a>` and `Term<'a>` utilize an invariant lifetime parameter:
```rust
type EnvId<'a> = PhantomData<*mut &'a u8>;
```
This prevents the Rust compiler from coercing `Env<'a>` into a different lifetime `'b`.

---

## Scheduling and Concurrency

### 1. Non-blocking Continuations (`NifOutcome`)

Long-running computations can yield back to the scheduler to maintain low BEAM latency:
1. `#[gleam_nif]` captures a function pointer and arguments in thread-local storage (`CURRENT_NIF_CONTINUATION`).
2. When the function returns `NifOutcome::Yield(flags)`, Gleamler invokes `enif_schedule_nif`.
3. The scheduler resumes execution in a fresh timeslice. From Gleam's perspective, the call is completely transparent and returns the final unwrapped value once `NifOutcome::Done(val)` is reached.

### 2. Dirty Schedulers

NIFs performing blocking I/O or heavy CPU work can be annotated with `#[gleam_nif(dirty_cpu)]` or `#[gleam_nif(dirty_io)]`. These functions execute on dedicated BEAM dirty scheduler thread pools without blocking normal schedulers.

### 3. Panic Boundary

All NIF calls are wrapped in `std::panic::catch_unwind(std::panic::AssertUnwindSafe(...))`:
- In standard mode: Panics are converted to an Erlang exception with captured backtrace information (`nif_panicked`).
- In safe mode (`#[gleam_nif(safe)]`): Panics and type decoding errors are captured and returned as a Gleam `Error(GleamlerError)`.

---

## OTP Integration and Messaging

### 1. Process Monitoring and Cancellation (`CancellationToken`)

Background OS threads can track the lifecycle of the calling Gleam process:
1. `env.cancellation_token()` creates a `CancellationResource` containing an `AtomicBool`.
2. It monitors the calling process via `enif_monitor_process`.
3. If the calling Gleam actor dies or is killed, BEAM invokes the resource's `down` callback:
   ```rust
   fn down<'a>(&'a self, _env: Env<'a>, _pid: LocalPid, _monitor: Monitor) {
       self.cancelled.store(true, Ordering::SeqCst);
   }
   ```
4. Background threads check `cancel_token.is_cancelled()` and terminate cleanly.

### 2. Zero-Allocation Messaging (`SubjectSender`)

Gleam actors receive messages through `process.Subject(T)`. In Erlang, this is represented as `{subject, Pid, Tag}`.
- `subject.to_sender()` captures the tag and pre-allocates message environments.
- In tight loops, `sender.send(msg)` utilizes `send_and_clear` (`enif_clear_env`), resetting heap pointers in-place without triggering `malloc` or `free` calls in the BEAM allocator.

---

## Verification and Quality Assurance

Gleamler is verified against multiple test layers:

- **Unit Tests:** Validates byte representation, ETF serialization, alignment and type conversions.
- **Garbage Collector Leak Tests (`resource_leak_test.erl`):** Allocates hundreds of megabytes in `ResourceArc` instances, forces 5 consecutive GC sweeps and asserts that memory delta remains within strict thresholds.
- **Valgrind Memcheck (`cargo xtask test --valgrind`):** Validates memory safety under Linux using `valgrind.supp` to filter benign thread-local allocations in the ERTS runtime.