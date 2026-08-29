import gleam/bool
import gleam/float
import gleam/int
import gleam/io
import gleam/string

import gleamler_nif
import stress_test
import vm

pub fn main() {
  io.println("=== Basic smoke test ===")
  io.println(
    "add(5, 10)        = " <> int.to_string(gleamler_nif.rust_add(5, 10)),
  )
  io.println(
    "sub(100, 7)       = " <> int.to_string(gleamler_nif.rust_sub(100, 7)),
  )
  io.println("greet(\"Gleam\")    = " <> gleamler_nif.rust_greet("Gleam"))

  let doubled = gleamler_nif.rust_double_list([1, 2, 3, 4, 5])
  io.println("double_list       = " <> string.inspect(doubled))

  io.println(
    "is_positive(5)    = " <> bool.to_string(gleamler_nif.rust_is_positive(5)),
  )
  io.println(
    "is_positive(-3)   = " <> bool.to_string(gleamler_nif.rust_is_positive(-3)),
  )

  io.println(
    "divide(10.0, 3.0) = "
    <> float.to_string(gleamler_nif.rust_divide(10.0, 3.0)),
  )

  let pair = gleamler_nif.rust_make_pair(42, "answer")
  io.println("make_pair         = " <> string.inspect(pair))

  io.println(
    "factorial(6)      = " <> int.to_string(gleamler_nif.rust_factorial(6)),
  )
  io.println("fib(8) dirty CPU  = " <> int.to_string(gleamler_nif.rust_fib(8)))

  io.println("\n=== i128 / u128 roundtrip ===")
  let i128_max = 170_141_183_460_469_231_731_687_303_715_884_105_727
  let i128_min = -170_141_183_460_469_231_731_687_303_715_884_105_728
  let u128_max = 340_282_366_920_938_463_463_374_607_431_768_211_455

  io.println(
    "echo_i128(MAX)    = "
    <> int.to_string(gleamler_nif.rust_echo_i128(i128_max)),
  )
  io.println(
    "echo_i128(MIN)    = "
    <> int.to_string(gleamler_nif.rust_echo_i128(i128_min)),
  )
  io.println(
    "echo_u128(MAX)    = "
    <> int.to_string(gleamler_nif.rust_echo_u128(u128_max)),
  )

  io.println("\n=== VM integration test ===")
  let program = [
    vm.Push(vm.VInt(5)),
    vm.Push(vm.VInt(10)),
    vm.Add,
    vm.Push(vm.VString("World")),
    vm.Greet,
    vm.Push(vm.VList([1, 2, 3])),
    vm.DoubleList,
    vm.Push(vm.VInt(42)),
    vm.Push(vm.VString("life")),
    vm.MakePair,
    vm.Push(vm.VInt(6)),
    vm.Factorial,
    vm.Push(vm.VInt(21)),
    vm.Fib,
    vm.Push(vm.VInt(170_141_183_460_469_231_731_687_303_715_884_105_727)),
    vm.EchoI128,
    vm.Push(vm.VInt(-170_141_183_460_469_231_731_687_303_715_884_105_728)),
    vm.EchoI128,
  ]

  case vm.run(program) {
    Ok(stack) -> io.println("VM final stack: " <> vm.inspect_stack(stack))
    Error(e) -> io.println("VM error: " <> e)
  }

  stress_test.run()
}
