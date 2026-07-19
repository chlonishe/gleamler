-module(gleamler_nif).
-export([add/2, init/0]).
-on_load(init/0).

init() ->
    {ok, CurrentDir} = file:get_cwd(),
    LibName = case os:type() of
        {win32, _} -> "gleamler";
        _ -> "libgleamler"
    end,
    Path = filename:join([CurrentDir, "target", "release", LibName]),
    
    case erlang:load_nif(Path, 0) of
        ok -> ok;
        Error -> 
            io:format("Error loading NIF: ~p~n", [Error]),
            Error
    end.

add(_A, _B) ->
    exit(nif_library_not_loaded).
