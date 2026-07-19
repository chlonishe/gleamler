-module(gleamler).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch, inline]).
-define(FILEPATH, "src\\gleamler.gleam").
-export([rust_add/2, main/0]).

-file("src\\gleamler.gleam", 5).
-spec rust_add(integer(), integer()) -> integer().
rust_add(A, B) ->
    gleamler_nif:add(A, B).

-file("src\\gleamler.gleam", 7).
-spec main() -> nil.
main() ->
    Result = gleamler_nif:add(5, 10),
    gleam_stdlib:println(
        <<"Real result from Rust: "/utf8,
            (erlang:integer_to_binary(Result))/binary>>
    ).
