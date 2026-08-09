-module(gleamler_nif_ffi).
-export([add/2, divide/2, double_list/1, echo_i128/1, echo_u128/1, factorial/1, fib/1, greet/1, is_positive/1, make_pair/2, mul/2, stress_add_wrap/2, stress_dirty_cpu/1, stress_dirty_io/1, stress_float_div/2, stress_float_is_special/1, stress_i128_max/0, stress_i128_min/0, stress_i64_max/0, stress_maybe_div/2, stress_mul_wrap/2, stress_now_ms/0, stress_panic/1, stress_repeat_string/2, stress_reverse_list/1, stress_safe_sqrt/1, stress_string_len/1, stress_sum_list/1, stress_tuple_swap/2, stress_u128_max/0, stress_u64_max/0, sub/2]).
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
add(_A, _B) -> exit(nif_library_not_loaded).
divide(_A, _B) -> exit(nif_library_not_loaded).
double_list(_A) -> exit(nif_library_not_loaded).
echo_i128(_A) -> exit(nif_library_not_loaded).
echo_u128(_A) -> exit(nif_library_not_loaded).
factorial(_A) -> exit(nif_library_not_loaded).
fib(_A) -> exit(nif_library_not_loaded).
greet(_A) -> exit(nif_library_not_loaded).
is_positive(_A) -> exit(nif_library_not_loaded).
make_pair(_A, _B) -> exit(nif_library_not_loaded).
mul(_A, _B) -> exit(nif_library_not_loaded).
stress_add_wrap(_A, _B) -> exit(nif_library_not_loaded).
stress_dirty_cpu(_A) -> exit(nif_library_not_loaded).
stress_dirty_io(_A) -> exit(nif_library_not_loaded).
stress_float_div(_A, _B) -> exit(nif_library_not_loaded).
stress_float_is_special(_A) -> exit(nif_library_not_loaded).
stress_i128_max() -> exit(nif_library_not_loaded).
stress_i128_min() -> exit(nif_library_not_loaded).
stress_i64_max() -> exit(nif_library_not_loaded).
stress_maybe_div(_A, _B) -> exit(nif_library_not_loaded).
stress_mul_wrap(_A, _B) -> exit(nif_library_not_loaded).
stress_now_ms() -> exit(nif_library_not_loaded).
stress_panic(_A) -> exit(nif_library_not_loaded).
stress_repeat_string(_A, _B) -> exit(nif_library_not_loaded).
stress_reverse_list(_A) -> exit(nif_library_not_loaded).
stress_safe_sqrt(_A) -> exit(nif_library_not_loaded).
stress_string_len(_A) -> exit(nif_library_not_loaded).
stress_sum_list(_A) -> exit(nif_library_not_loaded).
stress_tuple_swap(_A, _B) -> exit(nif_library_not_loaded).
stress_u128_max() -> exit(nif_library_not_loaded).
stress_u64_max() -> exit(nif_library_not_loaded).
sub(_A, _B) -> exit(nif_library_not_loaded).