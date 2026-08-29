use crate::gleam_nif;
use crate::{Resource, ResourceArc};
use std::time::Duration;

pub struct ValgrindTestResource {
    pub payload: Vec<u8>,
}

impl Resource for ValgrindTestResource {}

#[gleam_nif]
pub fn stress_i128_min() -> i128 {
    i128::MIN
}

#[gleam_nif]
pub fn stress_i128_max() -> i128 {
    i128::MAX
}

#[gleam_nif]
pub fn stress_u128_max() -> u128 {
    u128::MAX
}

#[gleam_nif]
pub fn stress_i64_max() -> i64 {
    i64::MAX
}

#[gleam_nif]
pub fn stress_u64_max() -> u64 {
    u64::MAX
}

#[gleam_nif]
pub fn stress_add_wrap(a: i64, b: i64) -> i64 {
    a.wrapping_add(b)
}

#[gleam_nif]
pub fn stress_mul_wrap(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}

#[gleam_nif]
pub fn stress_repeat_string(s: String, n: i64) -> String {
    if n < 0 {
        return String::new();
    }
    s.repeat(n as usize)
}

#[gleam_nif]
pub fn stress_string_len(s: String) -> i64 {
    s.len() as i64
}

#[gleam_nif]
pub fn stress_sum_list(items: Vec<i64>) -> i64 {
    items.iter().sum()
}

#[gleam_nif]
pub fn stress_reverse_list(items: Vec<i64>) -> Vec<i64> {
    items.into_iter().rev().collect()
}

#[gleam_nif]
pub fn stress_panic(_msg: String) -> i64 {
    panic!("intentional nif panic");
}

#[gleam_nif(dirty_cpu)]
pub fn stress_dirty_cpu(n: i64) -> i64 {
    let mut x = 1i64;
    for i in 0..n {
        x = x.wrapping_mul(i + 1).wrapping_add(1);
    }
    x
}

#[gleam_nif(dirty_io)]
pub fn stress_dirty_io(ms: i64) -> i64 {
    let safe_ms = ms.max(0);
    std::thread::sleep(Duration::from_millis(safe_ms as u64));
    safe_ms
}

#[gleam_nif]
pub fn stress_float_div(a: f64, b: f64) -> f64 {
    a / b
}

#[gleam_nif]
pub fn stress_tuple_swap(a: i64, b: String) -> (String, i64) {
    (b, a)
}

#[gleam_nif]
pub fn stress_maybe_div(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { None } else { Some(a / b) }
}

#[gleam_nif]
pub fn stress_safe_sqrt(n: f64) -> Result<f64, String> {
    if n < 0.0 {
        Err("negative".to_string())
    } else {
        Ok(n.sqrt())
    }
}

#[gleam_nif]
pub fn stress_now_ms() -> i64 {
    unsafe { crate::sys::enif_monotonic_time(crate::sys::ErlNifTimeUnit::ERL_NIF_MSEC) }
}

#[gleam_nif]
pub fn stress_float_is_special(n: f64) -> String {
    if n.is_nan() {
        "nan".to_string()
    } else if n.is_infinite() {
        "inf".to_string()
    } else if n == 0.0 && n.is_sign_negative() {
        "neg_zero".to_string()
    } else {
        "normal".to_string()
    }
}

#[gleam_nif]
pub fn stress_resource_roundtrip() -> ResourceArc<ValgrindTestResource> {
    ResourceArc::new(ValgrindTestResource {
        payload: vec![0xAB; 1024 * 1024],
    })
}

#[gleam_nif]
pub fn stress_resource_intentional_leak() {
    let res = ResourceArc::new(ValgrindTestResource {
        payload: vec![0xCD; 1024 * 1024],
    });
    std::mem::forget(res);
}
