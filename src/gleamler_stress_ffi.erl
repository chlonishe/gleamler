-module(gleamler_stress_ffi).
-export([rescue_panic/0, to_dynamic/1, test_process_monitor_cancellation/1]).

to_dynamic(X) -> X.

test_process_monitor_cancellation(CreateTokenFn) ->
    Parent = self(),
    _Child = spawn(fun() ->
        Token = CreateTokenFn(),
        Parent ! {token, Token}
    end),
    receive
        {token, Token} ->
            timer:sleep(50),
            Token
    after 1000 ->
        error(timeout)
    end.

rescue_panic() ->
    try gleamler_nif_ffi:stress_panic(<<"boom">>) of
        V -> {ok, V}
    catch
        error:{nif_panicked, _} ->
            {error, <<"nif_panicked">>};
        error:Reason ->
            {error, list_to_binary(io_lib:format("~p", [Reason]))}
    end.