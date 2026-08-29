import gleam/bool
import gleam/float
import gleam/int
import gleam/list
import gleam/string

import gleamler_nif

pub type Value {
  VInt(Int)
  VFloat(Float)
  VString(String)
  VBool(Bool)
  VList(List(Int))
  VPair(#(Int, String))
}

pub type Op {
  Push(Value)
  Add
  Sub
  Mul
  Div
  Greet
  DoubleList
  IsPositive
  MakePair
  Factorial
  Fib
  EchoI128
  EchoU128
}

pub fn run(program: List(Op)) -> Result(List(Value), String) {
  do_run(program, [])
}

fn do_run(ops: List(Op), stack: List(Value)) -> Result(List(Value), String) {
  case ops {
    [] -> Ok(stack)
    [op, ..rest] -> {
      case step(op, stack) {
        Ok(new_stack) -> do_run(rest, new_stack)
        Error(msg) -> Error(msg)
      }
    }
  }
}

fn step(op: Op, stack: List(Value)) -> Result(List(Value), String) {
  case op {
    Push(v) -> Ok([v, ..stack])

    Add -> {
      case stack {
        [VInt(b), VInt(a), ..rest] ->
          Ok([VInt(gleamler_nif.rust_add(a, b)), ..rest])
        _ -> Error("Add: need 2 ints")
      }
    }

    Sub -> {
      case stack {
        [VInt(b), VInt(a), ..rest] ->
          Ok([VInt(gleamler_nif.rust_sub(a, b)), ..rest])
        _ -> Error("Sub: need 2 ints")
      }
    }

    Mul -> {
      case stack {
        [VInt(b), VInt(a), ..rest] ->
          Ok([VInt(gleamler_nif.rust_mul(a, b)), ..rest])
        _ -> Error("Mul: need 2 ints")
      }
    }

    Div -> {
      case stack {
        [VFloat(b), VFloat(a), ..rest] ->
          Ok([VFloat(gleamler_nif.rust_divide(a, b)), ..rest])
        _ -> Error("Div: need 2 floats")
      }
    }

    Greet -> {
      case stack {
        [VString(name), ..rest] ->
          Ok([VString(gleamler_nif.rust_greet(name)), ..rest])
        _ -> Error("Greet: need string")
      }
    }

    DoubleList -> {
      case stack {
        [VList(items), ..rest] ->
          Ok([VList(gleamler_nif.rust_double_list(items)), ..rest])
        _ -> Error("DoubleList: need list")
      }
    }

    IsPositive -> {
      case stack {
        [VInt(n), ..rest] ->
          Ok([VBool(gleamler_nif.rust_is_positive(n)), ..rest])
        _ -> Error("IsPositive: need int")
      }
    }

    MakePair -> {
      case stack {
        [VString(s), VInt(i), ..rest] ->
          Ok([VPair(gleamler_nif.rust_make_pair(i, s)), ..rest])
        _ -> Error("MakePair: need int then string")
      }
    }

    Factorial -> {
      case stack {
        [VInt(n), ..rest] -> Ok([VInt(gleamler_nif.rust_factorial(n)), ..rest])
        _ -> Error("Factorial: need int")
      }
    }

    Fib -> {
      case stack {
        [VInt(n), ..rest] -> Ok([VInt(gleamler_nif.rust_fib(n)), ..rest])
        _ -> Error("Fib: need int")
      }
    }

    EchoI128 -> {
      case stack {
        [VInt(n), ..rest] -> Ok([VInt(gleamler_nif.rust_echo_i128(n)), ..rest])
        _ -> Error("EchoI128: need int")
      }
    }

    EchoU128 -> {
      case stack {
        [VInt(n), ..rest] -> Ok([VInt(gleamler_nif.rust_echo_u128(n)), ..rest])
        _ -> Error("EchoU128: need int")
      }
    }
  }
}

pub fn inspect_stack(stack: List(Value)) -> String {
  "["
  <> string.join(
    list.map(stack, fn(v) {
      case v {
        VInt(i) -> int.to_string(i)
        VFloat(f) -> float.to_string(f)
        VString(s) -> "\"" <> s <> "\""
        VBool(b) -> bool.to_string(b)
        VList(l) -> string.inspect(l)
        VPair(p) -> string.inspect(p)
      }
    }),
    ", ",
  )
  <> "]"
}
