import gleam/int
import gleam/io

@external(erlang, "gleamler_nif", "add")
pub fn rust_add(a: Int, b: Int) -> Int

pub fn main() {
  let result = rust_add(5, 10)

  io.println("Result: " <> int.to_string(result))
}
