//Tests

import gleam/int
import gleam/float
import gleam/io
import gleam/bool
import gleam/string

import gleamler_nif

pub fn main() {
  io.println("add(5, 10)        = " <> int.to_string(gleamler_nif.rust_add(5, 10)))
  io.println("sub(100, 7)       = " <> int.to_string(gleamler_nif.rust_sub(100, 7)))
  io.println("greet(\"Gleam\")    = " <> gleamler_nif.rust_greet("Gleam"))

  let doubled = gleamler_nif.rust_double_list([1, 2, 3, 4, 5])
  io.println("double_list       = " <> string.inspect(doubled))

  io.println("is_positive(5)    = " <> bool.to_string(gleamler_nif.rust_is_positive(5)))
  io.println("is_positive(-3)   = " <> bool.to_string(gleamler_nif.rust_is_positive(-3)))

  io.println("divide(10.0, 3.0) = " <> float.to_string(gleamler_nif.rust_divide(10.0, 3.0)))

  let pair = gleamler_nif.rust_make_pair(42, "answer")
  io.println("make_pair         = " <> string.inspect(pair))
}