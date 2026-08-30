import gleeunit
import gleeunit/should
import gleam/option
import gleamler_nif

@external(erlang, "gleamler_stress_ffi", "rescue_panic")
fn rescue_panic() -> Result(Int, String)

pub fn main() {
  gleeunit.main()
}

pub fn add_test() {
  gleamler_nif.rust_add(2, 3)
  |> should.equal(5)
}

pub fn sub_test() {
  gleamler_nif.rust_sub(10, 3)
  |> should.equal(7)
}

pub fn float_div_test() {
  gleamler_nif.rust_divide(7.0, 2.0)
  |> should.equal(3.5)
}

pub fn string_roundtrip_test() {
  gleamler_nif.rust_greet("World")
  |> should.equal("Hello, World!")
}

pub fn list_roundtrip_test() {
  gleamler_nif.rust_double_list([1, 2, 3])
  |> should.equal([2, 4, 6])
}

pub fn tuple_roundtrip_test() {
  gleamler_nif.rust_make_pair(42, "life")
  |> should.equal(#(42, "life"))
}

pub fn i128_max_test() {
  let max = 170_141_183_460_469_231_731_687_303_715_884_105_727
  gleamler_nif.rust_echo_i128(max)
  |> should.equal(max)
}

pub fn i128_min_test() {
  let min = -170_141_183_460_469_231_731_687_303_715_884_105_728
  gleamler_nif.rust_echo_i128(min)
  |> should.equal(min)
}

pub fn u128_max_test() {
  let max = 340_282_366_920_938_463_463_374_607_431_768_211_455
  gleamler_nif.rust_echo_u128(max)
  |> should.equal(max)
}

pub fn cooperative_yield_test() {
  let counter = gleamler_nif.rust_counter_new(2000)
  gleamler_nif.rust_cooperative_count(counter)
  |> should.equal(2000)
}

pub fn resource_read_test() {
  let counter = gleamler_nif.rust_counter_new(500)
  gleamler_nif.rust_counter_read(counter)
  |> should.equal(0)
}

pub fn option_some_test() {
  gleamler_nif.rust_stress_maybe_div(10.0, 2.0)
  |> should.equal(option.Some(5.0))
}

pub fn option_none_test() {
  gleamler_nif.rust_stress_maybe_div(10.0, 0.0)
  |> should.equal(option.None)
}

pub fn result_ok_test() {
  gleamler_nif.rust_stress_safe_sqrt(25.0)
  |> should.equal(Ok(5.0))
}

pub fn result_err_test() {
  gleamler_nif.rust_stress_safe_sqrt(-1.0)
  |> should.equal(Error("negative"))
}

pub fn panic_catch_test() {
  rescue_panic()
  |> should.equal(Error("nif_panicked"))
}
