-module(gleamler_stress_ffi).
-export([rescue_panic/0, to_dynamic/1]).

to_dynamic(X) -> X.

rescue_panic() ->
    try gleamler_nif_ffi:stress_panic(<<"boom">>) of
        V -> {ok, V}
    catch
        error:{nif_panicked, _} ->
            {error, <<"nif_panicked">>};
        error:Reason ->
            {error, list_to_binary(io_lib:format("~p", [Reason]))}
    end.