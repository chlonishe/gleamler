use crate::{gleam_nif, init_nifs};

#[gleam_nif]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[gleam_nif]
pub fn sub(a: i64, b: i64) -> i64 {
    a - b
}

#[gleam_nif]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[gleam_nif]
pub fn double_list(items: Vec<i64>) -> Vec<i64> {
    items.into_iter().map(|x| x * 2).collect()
}

#[gleam_nif]
pub fn is_positive(n: i64) -> bool {
    n > 0
}

#[gleam_nif]
pub fn divide(a: f64, b: f64) -> f64 {
    a / b
}

#[gleam_nif]
pub fn make_pair(a: i64, b: String) -> (i64, String) {
    (a, b)
}

#[gleam_nif]
pub fn factorial(n: i64) -> i64 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

#[gleam_nif(dirty_cpu)]
pub fn fib(n: i64) -> i64 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}

#[gleam_nif]
pub fn echo_i128(n: i128) -> i128 {
    n
}

#[gleam_nif]
pub fn echo_u128(n: u128) -> u128 {
    n
}

#[gleam_nif]
pub fn mul(a: i64, b: i64) -> i64 {
    a * b
}

#[doc(hidden)]
pub mod __generated_registry {
    include!(concat!(env!("OUT_DIR"), "/nif_registry.rs"));
}

init_nifs!();
