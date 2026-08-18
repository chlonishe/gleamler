-module(gleamler_stress_ffi).
-export([rescue_panic/0]).

rescue_panic() ->
    try gleamler_nif_ffi:stress_panic(<<"boom">>) of
        V -> {ok, V}
    catch
        error:nif_panicked ->
            {error, <<"nif_panicked">>};
        error:Reason ->
            {error, list_to_binary(io_lib:format("~p", [Reason]))}
    end.