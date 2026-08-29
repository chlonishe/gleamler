-module(resource_leak_test).
-export([run/0]).

-define(ITERATIONS, 100).
-define(LEAK_THRESHOLD_MB, 10).

run() ->
    io:format("=== Resource Leak Test ===~n"),
    io:format("Creating ~p resources (~p MB each) and discarding terms...~n",
              [?ITERATIONS, 1]),

    MemBefore = erlang:memory(total),
    io:format("Memory before: ~.2f MB~n", [MemBefore / 1048576]),

    _ = [gleamler_nif_ffi:stress_resource_roundtrip() || _ <- lists:seq(1, ?ITERATIONS)],

    [begin erlang:garbage_collect(), timer:sleep(50) end || _ <- lists:seq(1, 5)],

    MemAfter = erlang:memory(total),
    io:format("Memory after:  ~.2f MB~n", [MemAfter / 1048576]),

    DeltaMB = (MemAfter - MemBefore) / 1048576,
    io:format("Delta:         ~.2f MB~n", [DeltaMB]),

    if
        DeltaMB > ?LEAK_THRESHOLD_MB ->
            io:format("FAIL: Leak detected! ~.2f MB still held after GC.~n", [DeltaMB]),
            erlang:halt(1);
        true ->
            io:format("PASS: No leak (delta ~.2f MB within threshold).~n", [DeltaMB])
    end,

    io:format("=== Resource Leak Test Complete ===~n"),
    ok.
