import gleam/int
import gleam/io
import gleam/string
import gleam/list
import gleam/option
import gleamler_nif

@external(erlang, "gleamler_stress_ffi", "rescue_panic")
fn rescue_panic() -> Result(Int, String)

fn assert_eq(a: a, b: a, msg: String) {
  case a == b {
    True -> Nil
    False -> {
      io.println("ASSERT FAIL: " <> msg)
      io.println("  expected: " <> string.inspect(b))
      io.println("  actual:   " <> string.inspect(a))
      panic
    }
  }
}

fn section(name: String) {
  io.println("\n=== " <> name <> " ===")
}

fn make_range(start: Int, end: Int) -> List(Int) {
  do_make_range(start, end, [])
}

fn do_make_range(start: Int, end: Int, acc: List(Int)) -> List(Int) {
  case start > end {
    True -> list.reverse(acc)
    False -> do_make_range(start + 1, end, [start, ..acc])
  }
}

pub fn run() {
  section("Numeric boundaries")
  assert_eq(gleamler_nif.rust_stress_i128_min(), -170141183460469231731687303715884105728, "i128 min")
  assert_eq(gleamler_nif.rust_stress_i128_max(), 170141183460469231731687303715884105727, "i128 max")
  assert_eq(gleamler_nif.rust_stress_u128_max(), 340282366920938463463374607431768211455, "u128 max")
  assert_eq(gleamler_nif.rust_stress_i64_max(), 9223372036854775807, "i64 max")
  assert_eq(gleamler_nif.rust_stress_u64_max(), 18446744073709551615, "u64 max")
  assert_eq(gleamler_nif.rust_stress_add_wrap(9223372036854775807, 1), -9223372036854775808, "i64 wrap add")
  assert_eq(gleamler_nif.rust_stress_mul_wrap(9223372036854775807, 2), -2, "i64 wrap mul")
  io.println("PASS")

  section("String / Binary stress")
  let big = gleamler_nif.rust_stress_repeat_string("абв", 10000)
  assert_eq(string.length(big), 30000, "repeat grapheme len")
  assert_eq(gleamler_nif.rust_stress_string_len(big), 60000, "repeat byte len")
  io.println("PASS")

  section("List stress")
  let huge = make_range(1, 100_000)
  assert_eq(gleamler_nif.rust_stress_sum_list(huge), 5000050000, "sum 100k")
  let rev = gleamler_nif.rust_stress_reverse_list([1, 2, 3, 4, 5])
  assert_eq(rev, [5, 4, 3, 2, 1], "reverse list")
  io.println("PASS")

  section("Float precision & edge cases")
  assert_eq(gleamler_nif.rust_stress_float_div(1.0, 2.0), 0.5, "half")
  assert_eq(gleamler_nif.rust_stress_float_div(1.0, 4.0), 0.25, "quarter")
  assert_eq(gleamler_nif.rust_stress_float_div(22.0, 7.0), 3.142857142857143, "pi approx")
  assert_eq(gleamler_nif.rust_stress_float_div(-1.0, 1.0e308), -1.0e-308, "tiny neg")
  assert_eq(gleamler_nif.rust_stress_float_is_special(0.0), "normal", "zero is normal")
  assert_eq(gleamler_nif.rust_stress_float_is_special(-0.0), "neg_zero", "neg zero detected")
  io.println("PASS")

  section("Option / Result encoding")
  assert_eq(gleamler_nif.rust_stress_maybe_div(10.0, 2.0), option.Some(5.0), "some div")
  assert_eq(gleamler_nif.rust_stress_maybe_div(10.0, 0.0), option.None, "none div")
  assert_eq(gleamler_nif.rust_stress_safe_sqrt(16.0), Ok(4.0), "sqrt ok")
  assert_eq(gleamler_nif.rust_stress_safe_sqrt(-1.0), Error("negative"), "sqrt err")
  io.println("PASS")

  section("Tuple stress")
  assert_eq(gleamler_nif.rust_stress_tuple_swap(42, "hello"), #("hello", 42), "tuple swap")
  io.println("PASS")

    section("Panic recovery")
  assert_eq(
    rescue_panic(),
    Error("nif_panicked"),
    "panicking NIF raises nif_panicked",
  )
  assert_eq(
    gleamler_nif.rust_add(2, 3),
    5,
    "VM alive after NIF panic",
  )
  assert_eq(
    gleamler_nif.rust_greet("again"),
    "Hello, again!",
    "NIFs callable after panic",
  )
  io.println("PASS")

  section("Dirty CPU stress")
  let start = gleamler_nif.rust_stress_now_ms()
  let _ = gleamler_nif.rust_stress_dirty_cpu(10_000_000)
  let elapsed = gleamler_nif.rust_stress_now_ms() - start
  io.println("Dirty CPU 10M iterations took " <> int.to_string(elapsed) <> " ms")
  io.println("PASS")

  section("Dirty IO stress")
  let start = gleamler_nif.rust_stress_now_ms()
  let _ = gleamler_nif.rust_stress_dirty_io(150)
  let elapsed = gleamler_nif.rust_stress_now_ms() - start
  assert_eq(elapsed >= 100, True, "dirty io slept at least 100ms")
  io.println("Dirty IO 150ms took " <> int.to_string(elapsed) <> " ms")
  io.println("PASS")

  section("ALL STRESS TESTS PASSED")
}