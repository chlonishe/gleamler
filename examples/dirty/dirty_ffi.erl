-module(gleamler_nif_ffi).
-export([fib/1, sleep_ms/1, atom_or_string_to_string/1, get_map_field/2, identity/1]).
-on_load(init/0).
init() ->
    PrivDir = case code:which(?MODULE) of
        non_existing -> {ok, Cwd} = file:get_cwd(), filename:join(Cwd, "priv");
        BeamPath -> filename:join([filename:dirname(BeamPath), "..", "priv"])
    end,
    LibName = "dirty",
    Path = filename:join(PrivDir, LibName),
    case erlang:load_nif(Path, 0) of
        ok -> ok;
        Error -> io:format("[Gleamler NIF] Load error: ~p~n", [Error]), Error
    end.
atom_or_string_to_string(Term) when is_atom(Term) -> {ok, atom_to_binary(Term, utf8)};
atom_or_string_to_string(Term) when is_binary(Term) -> {ok, Term};
atom_or_string_to_string(_) -> {error, nil}.
identity(X) -> X.
get_map_field(Map, Key) when is_map(Map), is_binary(Key) ->
    case maps:find(Key, Map) of
        {ok, Val} -> {ok, Val};
        error ->
            try binary_to_existing_atom(Key, utf8) of
                AtomKey ->
                    case maps:find(AtomKey, Map) of
                        {ok, Val2} -> {ok, Val2};
                        error -> {error, missing_field}
                    end
            catch
                error:badarg -> {error, missing_field}
            end
    end;
get_map_field(_, _) -> {error, not_a_map}.
fib(_Arg0) -> exit(nif_library_not_loaded).
sleep_ms(_Arg0) -> exit(nif_library_not_loaded).