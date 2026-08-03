import gleam/int
import gleam/float
import gleam/io
import gleam/bool
import gleam/string

@external(erlang, "gleamler_nif", "add")
pub fn rust_add(a: Int, b: Int) -> Int

@external(erlang, "gleamler_nif", "sub")
pub fn rust_sub(a: Int, b: Int) -> Int

@external(erlang, "gleamler_nif", "greet")
pub fn rust_greet(name: String) -> String

@external(erlang, "gleamler_nif", "double_list")
pub fn rust_double_list(items: List(Int)) -> List(Int)

@external(erlang, "gleamler_nif", "is_positive")
pub fn rust_is_positive(n: Int) -> Bool

@external(erlang, "gleamler_nif", "divide")
pub fn rust_divide(a: Float, b: Float) -> Float

@external(erlang, "gleamler_nif", "make_pair")
pub fn rust_make_pair(a: Int, b: String) -> #(Int, String)

pub fn main() {
    
  io.println("add(5, 10)        = " <> int.to_string(rust_add(5, 10)))
  io.println("sub(100, 7)       = " <> int.to_string(rust_sub(100, 7)))
  io.println("greet(\"Gleam\")    = " <> rust_greet("Gleam"))

  let doubled = rust_double_list([1, 2, 3, 4, 5])
  io.println("double_list       = " <> string.inspect(doubled))

  io.println("is_positive(5)    = " <> bool.to_string(rust_is_positive(5)))
  io.println("is_positive(-3)   = " <> bool.to_string(rust_is_positive(-3)))

  io.println("divide(10.0, 3.0) = " <> float.to_string(rust_divide(10.0, 3.0)))

  let pair = rust_make_pair(42, "answer")
  io.println("make_pair         = " <> string.inspect(pair))
}