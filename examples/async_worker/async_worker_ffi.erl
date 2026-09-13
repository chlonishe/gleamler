-module(gleamler_nif_ffi).
-export([start_work/2]).
-on_load(init/0).
init() ->
    PrivDir = case code:which(?MODULE) of
        non_existing -> {ok, Cwd} = file:get_cwd(), filename:join(Cwd, "priv");
        BeamPath -> filename:join([filename:dirname(BeamPath), "..", "priv"])
    end,
    LibName = "async_worker",
    Path = filename:join(PrivDir, LibName),
    case erlang:load_nif(Path, 0) of
        ok -> ok;
        Error -> io:format("[Gleamler NIF] Load error: ~p~n", [Error]), Error
    end.
start_work(_Arg0, _Arg1) -> exit(nif_library_not_loaded).