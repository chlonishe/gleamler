-module(gleamler_nif_ffi).
-export([add/2, sub/2, greet/1, double_list/1, is_positive/1, divide/2, make_pair/2, factorial/1, fib/1]).

-on_load(init/0).

init() ->
    PrivDir = case code:which(?MODULE) of
        non_existing ->
            {ok, Cwd} = file:get_cwd(),
            filename:join(Cwd, "priv");
        BeamPath ->
            filename:join([filename:dirname(BeamPath), "..", "priv"])
    end,

    LibName = "gleamler",
    Path = filename:join(PrivDir, LibName),

    case erlang:load_nif(Path, 0) of
        ok -> ok;
        Error -> 
            io:format("[Gleamler NIF] Load error: ~p~n", [Error]),
            Error
    end.

add(_A, _B) -> exit(nif_library_not_loaded).
sub(_A, _B) -> exit(nif_library_not_loaded).
greet(_A) -> exit(nif_library_not_loaded).
double_list(_A) -> exit(nif_library_not_loaded).
is_positive(_A) -> exit(nif_library_not_loaded).
divide(_A, _B) -> exit(nif_library_not_loaded).
make_pair(_A, _B) -> exit(nif_library_not_loaded).
factorial(_A) -> exit(nif_library_not_loaded).
fib(_A) -> exit(nif_library_not_loaded).
