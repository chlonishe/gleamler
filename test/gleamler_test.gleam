import gleam/dict
import gleam/dynamic
import gleam/dynamic/decode
import gleam/int
import gleam/list
import gleam/option
import gleamler_nif
import gleeunit
import gleeunit/should

@external(erlang, "gleamler_stress_ffi", "rescue_panic")
fn rescue_panic() -> Result(Int, String)

@external(erlang, "gleamler_stress_ffi", "to_dynamic")
fn to_dynamic(a: a) -> dynamic.Dynamic

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

// collections

pub fn collections_hashset_roundtrip_test() {
  let result =
    gleamler_nif.rust_collections_hashset_roundtrip([1, 2, 3])
    |> list.sort(int.compare)
  result |> should.equal([1, 2, 3])
}

pub fn collections_btreeset_roundtrip_test() {
  gleamler_nif.rust_collections_btreeset_roundtrip(["c", "a", "b"])
  |> should.equal(["a", "b", "c"])
}

pub fn collections_vecdeque_roundtrip_test() {
  gleamler_nif.rust_collections_vecdeque_roundtrip([10, 20, 30])
  |> should.equal([10, 20, 30])
}

pub fn collections_linkedlist_roundtrip_test() {
  gleamler_nif.rust_collections_linkedlist_roundtrip([True, False, True])
  |> should.equal([True, False, True])
}

pub fn collections_btreemap_roundtrip_test() {
  let input = dict.from_list([#("x", 1), #("y", 2), #("z", 3)])
  gleamler_nif.rust_collections_btreemap_roundtrip(input)
  |> should.equal(input)
}

// net

pub fn net_ip_v4_roundtrip_test() {
  gleamler_nif.rust_net_ip_roundtrip("192.168.1.1")
  |> should.equal("192.168.1.1")
}

pub fn net_ip_v6_roundtrip_test() {
  gleamler_nif.rust_net_ip_roundtrip("::1")
  |> should.equal("::1")
}

pub fn net_socket_roundtrip_test() {
  gleamler_nif.rust_net_socket_roundtrip(#("127.0.0.1", 8080))
  |> should.equal(#("127.0.0.1", 8080))
}

pub fn net_socket_v4_roundtrip_test() {
  gleamler_nif.rust_net_socket_v4_roundtrip(#("8.8.8.8", 53))
  |> should.equal(#("8.8.8.8", 53))
}

pub fn net_socket_v6_roundtrip_test() {
  gleamler_nif.rust_net_socket_v6_roundtrip(#("::1", 443))
  |> should.equal(#("::1", 443))
}

// time

pub fn time_system_time_roundtrip_test() {
  gleamler_nif.rust_time_system_time_roundtrip(#(0, 1, 500_000))
  |> should.equal(#(0, 1, 500_000))
}

pub fn typegen_record_roundtrip_test() {
  let user = gleamler_nif.rust_make_user(1, "Alice")
  user.id |> should.equal(1)
  user.name |> should.equal("Alice")
  user.is_active |> should.equal(True)

  gleamler_nif.rust_user_get_name(user)
  |> should.equal("Alice")
}

pub fn typegen_unit_enum_test() {
  gleamler_nif.rust_is_user_banned(gleamler_nif.Banned)
  |> should.equal(True)

  gleamler_nif.rust_is_user_banned(gleamler_nif.Active)
  |> should.equal(False)
}

pub fn safe_div_success_test() {
  gleamler_nif.rust_safe_div(10.0, 2.0)
  |> should.equal(Ok(5.0))
}

pub fn safe_div_by_zero_test() {
  gleamler_nif.rust_safe_div(10.0, 0.0)
  |> should.equal(Error(gleamler_nif.Custom("division by zero")))
}

pub fn safe_panic_recovery_test() {
  gleamler_nif.rust_safe_panic_recovery(True)
  |> should.equal(Error(gleamler_nif.Panic("something went wrong in Rust!")))

  gleamler_nif.rust_safe_panic_recovery(False)
  |> should.equal(Ok(42))
}

pub fn decoder_record_tuple_test() {
  let user = gleamler_nif.rust_make_user(42, "Bob")
  let res = decode.run(to_dynamic(user), gleamler_nif.user_decoder())
  res |> should.equal(Ok(user))
}

pub fn decoder_record_map_test() {
  let map =
    dict.from_list([
      #("id", to_dynamic(99)),
      #("name", to_dynamic("Eve")),
      #("is_active", to_dynamic(False)),
    ])
  let res = decode.run(to_dynamic(map), gleamler_nif.user_decoder())
  res |> should.equal(Ok(gleamler_nif.User(99, "Eve", False)))
}

pub fn decoder_unit_enum_test() {
  let status = gleamler_nif.Active
  let res = decode.run(to_dynamic(status), gleamler_nif.user_status_decoder())
  res |> should.equal(Ok(gleamler_nif.Active))
}
