//! Dirty NIF examples.
//!
//! Long-running or blocking code must run outside the BEAM schedulers so it
//! does not degrade VM latency. Gleamler supports two dirty flags:
//!
//! * `dirty_cpu` — CPU-bound work (math, parsing, encoding). Runs in a
//!   limited pool of dirty-CPU threads.
//! * `dirty_io`  — IO-bound work (sleep, file/network ops). Runs in a
//!   separate dirty-IO thread pool.
use gleamler::{gleam_nif, init_nifs};
use std::thread;
use std::time::Duration;

/// Recursive Fibonacci. Exponential time — a classic `dirty_cpu` candidate.
#[gleam_nif(dirty_cpu)]
fn fib(n: i64) -> i64 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}

/// Sleeps the calling dirty-IO thread. Negative input is clamped to zero
/// to avoid a panic in `Duration::from_millis`.
#[gleam_nif(dirty_io)]
fn sleep_ms(ms: i64) -> i64 {
    let safe_ms = ms.max(0);
    thread::sleep(Duration::from_millis(safe_ms as u64));
    safe_ms
}

init_nifs!();
