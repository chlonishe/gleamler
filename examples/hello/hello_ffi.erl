-module(gleamler_nif_ffi).
-export([add/2, greet/1, atom_or_string_to_string/1, identity/1]).
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
atom_or_string_to_string(Term) when is_atom(Term) -> {ok, atom_to_binary(Term, utf8)};
atom_or_string_to_string(Term) when is_binary(Term) -> {ok, Term};
atom_or_string_to_string(_) -> {error, nil}.
identity(X) -> X.
add(_Arg0, _Arg1) -> exit(nif_library_not_loaded).
greet(_Arg0) -> exit(nif_library_not_loaded).