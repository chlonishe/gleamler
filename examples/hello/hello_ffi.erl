-module(gleamler_nif_ffi).
-export([add/2, greet/1]).
-on_load(init/0).
init() ->
    PrivDir = case code:which(?MODULE) of
        non_existing -> {ok, Cwd} = file:get_cwd(), filename:join(Cwd, "priv");
        BeamPath -> filename:join([filename:dirname(BeamPath), "..", "priv"])
    end,
    LibName = "hello",
    Path = filename:join(PrivDir, LibName),
    case erlang:load_nif(Path, 0) of
        ok -> ok;
        Error -> io:format("[Gleamler NIF] Load error: ~p~n", [Error]), Error
    end.
add(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
greet(_Arg0) -> exit(nif_library_not_loaded).