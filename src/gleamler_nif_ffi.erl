-module(gleamler_nif_ffi).
-export([counter_new/1, cooperative_count/1, add/2, sub/2, greet/1, double_list/1, is_positive/1, divide/2, make_pair/2, factorial/1, fib/1, echo_i128/1, echo_u128/1, mul/2, stress_i128_min/0, stress_i128_max/0, stress_u128_max/0, stress_i64_max/0, stress_u64_max/0, stress_add_wrap/2, stress_mul_wrap/2, stress_repeat_string/2, stress_string_len/1, stress_sum_list/1, stress_reverse_list/1, stress_panic/1, stress_dirty_cpu/1, stress_dirty_io/1, stress_float_div/2, stress_tuple_swap/2, stress_maybe_div/2, stress_safe_sqrt/1, stress_now_ms/0, stress_float_is_special/1, stress_resource_roundtrip/0, stress_resource_intentional_leak/0]).
-on_load(init/0).
init() ->
    PrivDir = case code:which(?MODULE) of
        non_existing -> {ok, Cwd} = file:get_cwd(), filename:join(Cwd, "priv");
        BeamPath -> filename:join([filename:dirname(BeamPath), "..", "priv"])
    end,
    LibName = "gleamler",
    Path = filename:join(PrivDir, LibName),
    case erlang:load_nif(Path, 0) of
        ok -> ok;
        Error -> io:format("[Gleamler NIF] Load error: ~p~n", [Error]), Error
    end.
counter_new(_Arg0) -> exit(nif_library_not_loaded).
cooperative_count(_Arg0) -> exit(nif_library_not_loaded).
add(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
sub(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
greet(_Arg0) -> exit(nif_library_not_loaded).
double_list(_Arg0) -> exit(nif_library_not_loaded).
is_positive(_Arg0) -> exit(nif_library_not_loaded).
divide(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
make_pair(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
factorial(_Arg0) -> exit(nif_library_not_loaded).
fib(_Arg0) -> exit(nif_library_not_loaded).
echo_i128(_Arg0) -> exit(nif_library_not_loaded).
echo_u128(_Arg0) -> exit(nif_library_not_loaded).
mul(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
stress_i128_min() -> exit(nif_library_not_loaded).
stress_i128_max() -> exit(nif_library_not_loaded).
stress_u128_max() -> exit(nif_library_not_loaded).
stress_i64_max() -> exit(nif_library_not_loaded).
stress_u64_max() -> exit(nif_library_not_loaded).
stress_add_wrap(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
stress_mul_wrap(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
stress_repeat_string(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
stress_string_len(_Arg0) -> exit(nif_library_not_loaded).
stress_sum_list(_Arg0) -> exit(nif_library_not_loaded).
stress_reverse_list(_Arg0) -> exit(nif_library_not_loaded).
stress_panic(_Arg0) -> exit(nif_library_not_loaded).
stress_dirty_cpu(_Arg0) -> exit(nif_library_not_loaded).
stress_dirty_io(_Arg0) -> exit(nif_library_not_loaded).
stress_float_div(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
stress_tuple_swap(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
stress_maybe_div(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
stress_safe_sqrt(_Arg0) -> exit(nif_library_not_loaded).
stress_now_ms() -> exit(nif_library_not_loaded).
stress_float_is_special(_Arg0) -> exit(nif_library_not_loaded).
stress_resource_roundtrip() -> exit(nif_library_not_loaded).
stress_resource_intentional_leak() -> exit(nif_library_not_loaded).