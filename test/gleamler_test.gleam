import gleeunit
import gleeunit/should
import gleam/option

import gleamler_nif

pub fn main() {
  gleeunit.main()
}

fn make_range(start: Int, end: Int) -> List(Int) {
  case start > end {
    True -> []
    False -> [start, ..make_range(start + 1, end)]
  }
}

pub fn add_test() {
  gleamler_nif.rust_add(2, 3) |> should.equal(5)
}

pub fn sub_test() {
  gleamler_nif.rust_sub(100, 7) |> should.equal(93)
}

pub fn mul_test() {
  gleamler_nif.rust_mul(6, 7) |> should.equal(42)
}

pub fn greet_test() {
  gleamler_nif.rust_greet("Gleam")
  |> should.equal("Hello, Gleam!")
}

pub fn repeat_string_test() {
  gleamler_nif.rust_stress_repeat_string("ha", 3)
  |> should.equal("hahaha")
}

pub fn double_list_test() {
  gleamler_nif.rust_double_list([1, 2, 3])
  |> should.equal([2, 4, 6])
}

pub fn reverse_list_test() {
  gleamler_nif.rust_stress_reverse_list([1, 2, 3, 4, 5])
  |> should.equal([5, 4, 3, 2, 1])
}

pub fn sum_list_test() {
  let nums = make_range(1, 100)
  gleamler_nif.rust_stress_sum_list(nums)
  |> should.equal(5050)
}

pub fn empty_list_test() {
  gleamler_nif.rust_double_list([]) |> should.equal([])
}

pub fn is_positive_true_test() {
  gleamler_nif.rust_is_positive(42) |> should.be_true
}

pub fn is_positive_false_test() {
  gleamler_nif.rust_is_positive(-5) |> should.be_false
}

pub fn divide_test() {
  gleamler_nif.rust_divide(10.0, 4.0)
  |> should.equal(2.5)
}

pub fn float_div_exact_test() {
  gleamler_nif.rust_stress_float_div(1.0, 2.0)
  |> should.equal(0.5)
}

pub fn make_pair_test() {
  gleamler_nif.rust_make_pair(42, "life")
  |> should.equal(#(42, "life"))
}

pub fn tuple_swap_test() {
  gleamler_nif.rust_stress_tuple_swap(1, "a")
  |> should.equal(#("a", 1))
}

pub fn factorial_test() {
  gleamler_nif.rust_factorial(6) |> should.equal(720)
}

pub fn fib_test() {
  gleamler_nif.rust_fib(10) |> should.equal(55)
}

pub fn i128_max_test() {
  let max = 170141183460469231731687303715884105727
  gleamler_nif.rust_echo_i128(max) |> should.equal(max)
}

pub fn i128_min_test() {
  let min = -170141183460469231731687303715884105728
  gleamler_nif.rust_echo_i128(min) |> should.equal(min)
}

pub fn u128_max_test() {
  let max = 340282366920938463463374607431768211455
  gleamler_nif.rust_echo_u128(max) |> should.equal(max)
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
  gleamler_nif.rust_stress_safe_sqrt(-4.0)
  |> should.equal(Error("negative"))
}

pub fn zero_add_test() {
  gleamler_nif.rust_add(0, 0) |> should.equal(0)
}

pub fn large_number_test() {
  gleamler_nif.rust_stress_i64_max()
  |> should.equal(9_223_372_036_854_775_807)
}