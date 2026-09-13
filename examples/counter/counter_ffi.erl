-module(gleamler_nif_ffi).
-export([counter_new/0, counter_inc/1, counter_get/1]).
-on_load(init/0).
init() ->
    PrivDir = case code:which(?MODULE) of
        non_existing -> {ok, Cwd} = file:get_cwd(), filename:join(Cwd, "priv");
        BeamPath -> filename:join([filename:dirname(BeamPath), "..", "priv"])
    end,
    LibName = "counter",
    Path = filename:join(PrivDir, LibName),
    case erlang:load_nif(Path, 0) of
        ok -> ok;
        Error -> io:format("[Gleamler NIF] Load error: ~p~n", [Error]), Error
    end.
counter_new() -> exit(nif_library_not_loaded).
counter_inc(_Arg0) -> exit(nif_library_not_loaded).
counter_get(_Arg0) -> exit(nif_library_not_loaded).