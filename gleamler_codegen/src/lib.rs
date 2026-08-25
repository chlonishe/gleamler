use proc_macro2::{Delimiter, TokenTree};
use syn::{
    AngleBracketedGenericArguments, FnArg, GenericArgument, Item, ItemFn, Pat, PathArguments,
    ReturnType, Type, TypeArray, TypePath, TypeReference, TypeSlice, TypeTuple, parse_file,
};

#[derive(Debug)]
pub struct NifFunc {
    pub name: String,
    pub alias: Option<String>,
    pub args: Vec<(String, String)>,
    pub ret: String,
    pub arity: usize,
    pub docs: Vec<String>,
}

pub fn parse_nif_functions(source: &str) -> Vec<NifFunc> {
    let file = parse_file(source).expect("failed to parse NIF source file");
    let mut functions = Vec::new();
    for item in file.items {
        if let syn::Item::Fn(func) = item
            && has_gleam_nif(&func.attrs)
            && let Some(f) = parse_nif_function(func)
        {
            functions.push(f);
        }
    }
    functions
}

pub fn parse_init_nifs_list(source: &str) -> Vec<String> {
    let Ok(file) = parse_file(source) else {
        return Vec::new();
    };
    for item in file.items {
        let Item::Macro(m) = item else { continue };
        if !m.mac.path.is_ident("init_nifs") {
            continue;
        }
        for tt in m.mac.tokens.clone() {
            let TokenTree::Group(g) = tt else { continue };
            if g.delimiter() != Delimiter::Bracket {
                continue;
            }
            return g
                .stream()
                .into_iter()
                .filter_map(|t| match t {
                    TokenTree::Ident(id) => Some(id.to_string()),
                    _ => None,
                })
                .collect();
        }
    }
    Vec::new()
}

fn has_gleam_nif(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        let path = a.path();
        path.is_ident("gleam_nif")
            || path
                .segments
                .last()
                .is_some_and(|seg| seg.ident == "gleam_nif")
    })
}

fn is_env_type(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            path.segments.last().is_some_and(|seg| seg.ident == "Env")
        }
        Type::Reference(TypeReference { elem, .. }) => is_env_type(elem),
        _ => false,
    }
}

#[cfg(test)]
fn type_to_gleam(ty: &Type) -> String {
    type_to_gleam_ctx(ty, "<unknown>")
}

fn type_to_gleam_ctx(ty: &Type, ctx: &str) -> String {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let segment = match path.segments.last() {
                Some(seg) => seg,
                None => return "Nil".into(),
            };
            let ident_str = segment.ident.to_string();

            let generic_args: Vec<&Type> = match &segment.arguments {
                PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => args
                    .iter()
                    .filter_map(|arg| {
                        if let GenericArgument::Type(t) = arg {
                            Some(t)
                        } else {
                            None
                        }
                    })
                    .collect(),
                _ => vec![],
            };

            match ident_str.as_str() {
                "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize"
                | "i128" | "u128" => "Int".into(),

                "f32" | "f64" => "Float".into(),

                "bool" => "Bool".into(),

                "String" | "str" => "String".into(),

                "Atom" => panic!(
                    "gleamler_codegen: bare Atom type is not supported in #[gleam_nif] fn '{ctx}'. \
                    Gleam has no built-in Atom type — use String instead"
                ),

                "Binary" | "OwnedBinary" | "NewBinary" => "BitArray".into(),

                "Vec" => {
                    if let Some(inner) = generic_args.first() {
                        format!("List({})", type_to_gleam_ctx(inner, ctx))
                    } else {
                        "List(Nil)".into()
                    }
                }

                "Option" => {
                    if let Some(inner) = generic_args.first() {
                        format!("option.Option({})", type_to_gleam_ctx(inner, ctx))
                    } else {
                        "option.Option(Nil)".into()
                    }
                }

                "Result" => match generic_args.len() {
                    1 => format!("Result({}, Nil)", type_to_gleam_ctx(generic_args[0], ctx)),
                    2 => format!(
                        "Result({}, {})",
                        type_to_gleam_ctx(generic_args[0], ctx),
                        type_to_gleam_ctx(generic_args[1], ctx)
                    ),
                    _ => "Result(Nil, Nil)".into(),
                },

                "HashMap" | "BTreeMap" => {
                    if generic_args.len() == 2 {
                        format!(
                            "dict.Dict({}, {})",
                            type_to_gleam_ctx(generic_args[0], ctx),
                            type_to_gleam_ctx(generic_args[1], ctx)
                        )
                    } else {
                        "dict.Dict(Nil, Nil)".into()
                    }
                }

                "ResourceArc" => "Resource".into(),

                "NifOutcome" => {
                    if let Some(inner) = generic_args.first() {
                        type_to_gleam_ctx(inner, ctx)
                    } else {
                        "Nil".into()
                    }
                }

                _ => {
                    if generic_args.is_empty() {
                        ident_str
                    } else {
                        let args_str: Vec<_> = generic_args
                            .iter()
                            .map(|t| type_to_gleam_ctx(t, ctx))
                            .collect();
                        format!("{}({})", ident_str, args_str.join(", "))
                    }
                }
            }
        }

        Type::Reference(TypeReference { elem, .. }) => type_to_gleam_ctx(elem, ctx),

        Type::Tuple(TypeTuple { elems, .. }) => {
            if elems.is_empty() {
                "Nil".into()
            } else {
                let parts: Vec<_> = elems.iter().map(|t| type_to_gleam_ctx(t, ctx)).collect();
                format!("#({})", parts.join(", "))
            }
        }

        Type::Slice(TypeSlice { elem, .. }) => {
            format!("List({})", type_to_gleam_ctx(elem, ctx))
        }

        Type::Array(TypeArray { elem, .. }) => {
            format!("List({})", type_to_gleam_ctx(elem, ctx))
        }

        _ => {
            use quote::ToTokens;
            ty.to_token_stream()
                .to_string()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("")
        }
    }
}

fn parse_nif_function(func: ItemFn) -> Option<NifFunc> {
    let name = func.sig.ident.to_string();
    let mut args = Vec::new();
    for (idx, arg) in func.sig.inputs.iter().enumerate() {
        if let FnArg::Typed(pat_type) = arg {
            if is_env_type(&pat_type.ty) {
                continue;
            }
            let arg_name = match pat_type.pat.as_ref() {
                Pat::Ident(ident) => ident.ident.to_string(),
                _ => format!("arg{}", idx),
            };
            let ty = type_to_gleam_ctx(&pat_type.ty, &name);
            args.push((arg_name, ty));
        }
    }
    let ret = match &func.sig.output {
        ReturnType::Default => "Nil".into(),
        ReturnType::Type(_, ty) => type_to_gleam_ctx(ty, &name),
    };
    let alias = func
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("gleam_nif"))
        .find_map(|a| {
            if let syn::Meta::List(list) = &a.meta {
                list.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                )
                .ok()
            } else {
                None
            }
        })
        .and_then(|metas| {
            metas.iter().find_map(|meta| {
                if meta.path().is_ident("alias") {
                    meta.require_name_value().ok().and_then(|nv| {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }) = &nv.value
                        {
                            Some(s.value())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            })
        });
    let arity = args.len();

    let docs: Vec<String> = func
        .attrs
        .iter()
        .filter_map(|a| {
            if a.path().is_ident("doc")
                && let syn::Meta::NameValue(nv) = &a.meta
                && let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
            {
                let text = s.value();
                let trimmed = text.strip_prefix(' ').unwrap_or(&text);
                return Some(trimmed.to_string());
            }
            None
        })
        .collect();

    Some(NifFunc {
        name,
        alias,
        args,
        ret,
        arity,
        docs,
    })
}

pub fn generate_erl(funcs: &[NifFunc], erl_module: &str, lib_name: &str) -> String {
    let exports: Vec<_> = funcs
        .iter()
        .map(|f| {
            let name = f.alias.as_ref().unwrap_or(&f.name);
            format!("{}/{}", name, f.arity)
        })
        .collect();
    let stubs: Vec<_> = funcs
        .iter()
        .map(|f| {
            let name = f.alias.as_ref().unwrap_or(&f.name);
            let args: Vec<_> = (0..f.arity).map(|i| format!("_Arg{}", i)).collect();
            format!(
                "{}({}) -> exit(nif_library_not_loaded).",
                name,
                args.join(", ")
            )
        })
        .collect();
    format!(
        r#"-module({erl_module}).
-export([{}]).
-on_load(init/0).
init() ->
    PrivDir = case code:which(?MODULE) of
        non_existing -> {{ok, Cwd}} = file:get_cwd(), filename:join(Cwd, "priv");
        BeamPath -> filename:join([filename:dirname(BeamPath), "..", "priv"])
    end,
    LibName = "{lib_name}",
    Path = filename:join(PrivDir, LibName),
    case erlang:load_nif(Path, 0) of
        ok -> ok;
        Error -> io:format("[Gleamler NIF] Load error: ~p~n", [Error]), Error
    end.
{}"#,
        exports.join(", "),
        stubs.join("\n")
    )
}

const GLEAM_KEYWORDS: &[&str] = &[
    "as",
    "assert",
    "auto",
    "case",
    "const",
    "delegate",
    "derive",
    "echo",
    "else",
    "fn",
    "if",
    "implement",
    "import",
    "let",
    "macro",
    "opaque",
    "panic",
    "pub",
    "test",
    "todo",
    "type",
    "use",
];

pub fn generate_gleam(funcs: &[NifFunc], erl_module: &str) -> String {
    let mut has_option = false;
    let mut has_dict = false;
    let mut has_resource = false;

    for f in funcs {
        if f.ret.contains("option.Option")
            || f.args.iter().any(|(_, t)| t.contains("option.Option"))
        {
            has_option = true;
        }
        if f.ret.contains("dict.Dict") || f.args.iter().any(|(_, t)| t.contains("dict.Dict")) {
            has_dict = true;
        }
        if gleam_type_mentions(&f.ret, "Resource")
            || f.args
                .iter()
                .any(|(_, t)| gleam_type_mentions(t, "Resource"))
        {
            has_resource = true;
        }
    }

    let mut out = String::from("// AUTOGENERATED by gleamler_codegen\n// Do not edit\n\n");
    if has_option {
        out.push_str("import gleam/option\n");
    }
    if has_dict {
        out.push_str("import gleam/dict\n");
    }
    if has_resource {
        out.push_str(
            "\npub opaque type Resource {\n  Resource\n}\n\n\
            @internal\npub fn resource_dummy() -> Resource {\n  Resource\n}\n",
        );
    }
    out.push('\n');

    for f in funcs {
        if !f.docs.is_empty() {
            out.push_str("/// ");
            out.push_str(&f.docs.join("\n/// "));
            out.push('\n');
        }
        let gleam_name = format!("rust_{}", f.alias.as_ref().unwrap_or(&f.name));
        let erl_name = f.alias.as_ref().unwrap_or(&f.name);
        let args: Vec<_> = f
            .args
            .iter()
            .map(|(n, t)| format!("{}: {}", clean_name(n), t))
            .collect();
        let ret = &f.ret;
        out.push_str(&format!(
            "@external(erlang, \"{erl_module}\", \"{}\")\npub fn {}({}) -> {}\n",
            erl_name,
            gleam_name,
            args.join(", "),
            ret
        ));
    }
    out
}

fn clean_name(n: &str) -> String {
    let name = n.trim_start_matches('_');
    let name = name.strip_prefix("r#").unwrap_or(name);
    let name = if name.is_empty() { "arg" } else { name };
    if GLEAM_KEYWORDS.contains(&name) {
        format!("{}_", name)
    } else {
        name.to_string()
    }
}

fn gleam_type_mentions(gleam_ty: &str, name: &str) -> bool {
    let chars = gleam_ty.chars();
    let mut buf = String::new();
    for c in chars {
        if c.is_alphanumeric() || c == '_' {
            buf.push(c);
        } else {
            if buf == name {
                return true;
            }
            buf.clear();
        }
    }
    buf == name
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    const DEFAULT_ERL_MODULE: &str = "gleamler_nif_ffi";
    const DEFAULT_LIB_NAME: &str = "gleamler";

    #[test]
    fn parse_empty_file() {
        assert!(parse_nif_functions("// nothing here").is_empty());
    }

    #[test]
    fn parse_simple_fn() {
        let src = r#"
#[gleam_nif]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}
"#;
        let funcs = parse_nif_functions(src);
        assert_eq!(funcs.len(), 1);
        let f = &funcs[0];
        assert_eq!(f.name, "add");
        assert_eq!(f.arity, 2);
        assert_eq!(
            f.args,
            vec![("a".into(), "Int".into()), ("b".into(), "Int".into())]
        );
        assert_eq!(f.ret, "Int");
    }

    #[test]
    fn parse_env_skipped() {
        let src = r#"
#[gleam_nif]
pub fn foo(env: Env, x: i64, name: String) -> bool {
    true
}
"#;
        let funcs = parse_nif_functions(src);
        assert_eq!(funcs[0].arity, 2);
        assert_eq!(funcs[0].args[0].0, "x");
        assert_eq!(funcs[0].args[1].0, "name");
        assert_eq!(funcs[0].args[0].1, "Int");
        assert_eq!(funcs[0].args[1].1, "String");
        assert_eq!(funcs[0].ret, "Bool");
    }

    #[test]
    fn parse_dirty_attributes() {
        let src = r#"
#[gleam_nif(dirty_cpu)]
pub fn heavy(n: i64) -> i64 { n }
"#;
        let funcs = parse_nif_functions(src);
        assert_eq!(funcs[0].name, "heavy");
    }

    #[test]
    fn erl_output_smoke() {
        let funcs = vec![NifFunc {
            name: "add".into(),
            alias: None,
            args: vec![("a".into(), "Int".into()), ("b".into(), "Int".into())],
            ret: "Int".into(),
            arity: 2,
            docs: vec![],
        }];
        let out = generate_erl(&funcs, DEFAULT_ERL_MODULE, DEFAULT_LIB_NAME);
        assert!(out.contains("-export([add/2])."));
        assert!(out.contains("add(_Arg0, _Arg1) -> exit(nif_library_not_loaded)."));
        assert!(out.contains("-module(gleamler_nif_ffi)."));
    }

    #[test]
    fn erl_output_custom_module_and_lib() {
        let funcs = vec![NifFunc {
            name: "hello".into(),
            alias: None,
            args: vec![],
            ret: "String".into(),
            arity: 0,
            docs: vec![],
        }];
        let out = generate_erl(&funcs, "my_custom_ffi", "my_lib");
        assert!(out.contains("-module(my_custom_ffi)."));
        assert!(out.contains("LibName = \"my_lib\","));
        assert!(out.contains("hello() -> exit(nif_library_not_loaded)."));
    }

    #[test]
    fn gleam_output_basic() {
        let funcs = vec![NifFunc {
            name: "greet".into(),
            alias: None,
            args: vec![("name".into(), "String".into())],
            ret: "String".into(),
            arity: 1,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, DEFAULT_ERL_MODULE);
        assert!(out.contains(r#"@external(erlang, "gleamler_nif_ffi", "greet")"#));
        assert!(out.contains("pub fn rust_greet(name: String) -> String"));
    }

    #[test]
    fn gleam_output_custom_erl_module() {
        let funcs = vec![NifFunc {
            name: "ping".into(),
            alias: None,
            args: vec![],
            ret: "Nil".into(),
            arity: 0,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, "other_ffi_module");
        assert!(out.contains(r#"@external(erlang, "other_ffi_module", "ping")"#));
    }

    #[test]
    fn gleam_output_complex_types() {
        let funcs = vec![NifFunc {
            name: "calc".into(),
            alias: None,
            args: vec![
                ("items".into(), "List(Int)".into()),
                ("flag".into(), "option.Option(Bool)".into()),
            ],
            ret: "Result(String, Int)".into(),
            arity: 2,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, DEFAULT_ERL_MODULE);
        assert!(out.contains("import gleam/option"));
        assert!(out.contains("items: List(Int)"));
        assert!(out.contains("flag: option.Option(Bool)"));
        assert!(out.contains("Result(String, Int)"));
    }

    #[test]
    fn gleam_tuple_type() {
        let funcs = vec![NifFunc {
            name: "pair".into(),
            alias: None,
            args: vec![("a".into(), "Int".into()), ("b".into(), "String".into())],
            ret: "#(Int, String)".into(),
            arity: 2,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, DEFAULT_ERL_MODULE);
        assert!(out.contains("#(Int, String)"));
    }

    #[test]
    fn gleam_dict_type() {
        let funcs = vec![NifFunc {
            name: "m".into(),
            alias: None,
            args: vec![("x".into(), "dict.Dict(String, Int)".into())],
            ret: "Nil".into(),
            arity: 1,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, DEFAULT_ERL_MODULE);
        assert!(out.contains("dict.Dict(String, Int)"));
    }

    #[test]
    fn gleam_vec_u8_maps_to_list_int() {
        let funcs = vec![NifFunc {
            name: "bytes".into(),
            alias: None,
            args: vec![("data".into(), "List(Int)".into())],
            ret: "List(Int)".into(),
            arity: 1,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, DEFAULT_ERL_MODULE);
        assert!(out.contains("data: List(Int)"));
        assert!(out.contains("-> List(Int)"));
        assert!(!out.contains("BitArray"));
    }

    #[test]
    fn gleam_slice_maps_to_list() {
        let funcs = vec![NifFunc {
            name: "s".into(),
            alias: None,
            args: vec![("data".into(), "List(Int)".into())],
            ret: "Nil".into(),
            arity: 1,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, DEFAULT_ERL_MODULE);
        assert!(out.contains("data: List(Int)"));
    }

    #[test]
    fn gleam_lifetimes_in_references() {
        let funcs = vec![NifFunc {
            name: "f".into(),
            alias: None,
            args: vec![
                ("s".into(), "String".into()),
                ("b".into(), "List(Int)".into()),
            ],
            ret: "Nil".into(),
            arity: 2,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, DEFAULT_ERL_MODULE);
        assert!(out.contains("s: String"));
        assert!(out.contains("b: List(Int)"));
    }

    #[test]
    fn gleam_lifetimes_in_binary() {
        let funcs = vec![NifFunc {
            name: "f".into(),
            alias: None,
            args: vec![("b".into(), "BitArray".into())],
            ret: "BitArray".into(),
            arity: 1,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, DEFAULT_ERL_MODULE);
        assert!(out.contains("b: BitArray"));
        assert!(out.contains("-> BitArray"));
    }

    #[test]
    fn gleam_lifetimes_in_complex_generics() {
        let funcs = vec![NifFunc {
            name: "f".into(),
            alias: None,
            args: vec![
                ("x".into(), "Foo(T)".into()),
                ("y".into(), "Bar(T, U)".into()),
            ],
            ret: "Nil".into(),
            arity: 2,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, DEFAULT_ERL_MODULE);
        assert!(out.contains("x: Foo(T)"));
        assert!(out.contains("y: Bar(T, U)"));
    }

    #[test]
    fn gleam_tuple_with_nested_generics() {
        let funcs = vec![NifFunc {
            name: "complex".into(),
            alias: None,
            args: vec![],
            ret: "#(dict.Dict(String, Int), Bool)".into(),
            arity: 0,
            docs: vec![],
        }];
        let out = generate_gleam(&funcs, DEFAULT_ERL_MODULE);
        assert!(out.contains("#(dict.Dict(String, Int), Bool)"));
    }

    #[test]
    fn type_to_gleam_primitives() {
        let ty: Type = parse_quote!(i32);
        assert_eq!(type_to_gleam(&ty), "Int");
        let ty: Type = parse_quote!(u64);
        assert_eq!(type_to_gleam(&ty), "Int");
        let ty: Type = parse_quote!(i128);
        assert_eq!(type_to_gleam(&ty), "Int");
        let ty: Type = parse_quote!(u128);
        assert_eq!(type_to_gleam(&ty), "Int");
        let ty: Type = parse_quote!(usize);
        assert_eq!(type_to_gleam(&ty), "Int");
        let ty: Type = parse_quote!(f32);
        assert_eq!(type_to_gleam(&ty), "Float");
        let ty: Type = parse_quote!(f64);
        assert_eq!(type_to_gleam(&ty), "Float");
        let ty: Type = parse_quote!(bool);
        assert_eq!(type_to_gleam(&ty), "Bool");
        let ty: Type = parse_quote!(String);
        assert_eq!(type_to_gleam(&ty), "String");
    }

    #[test]
    fn type_to_gleam_generics() {
        let ty: Type = parse_quote!(Vec<i32>);
        assert_eq!(type_to_gleam(&ty), "List(Int)");
        let ty: Type = parse_quote!(Option<String>);
        assert_eq!(type_to_gleam(&ty), "option.Option(String)");
        let ty: Type = parse_quote!(Result<i32, String>);
        assert_eq!(type_to_gleam(&ty), "Result(Int, String)");
        let ty: Type = parse_quote!(HashMap<String, i32>);
        assert_eq!(type_to_gleam(&ty), "dict.Dict(String, Int)");
    }

    #[test]
    fn type_to_gleam_tuples() {
        let ty: Type = parse_quote!((i32, String));
        assert_eq!(type_to_gleam(&ty), "#(Int, String)");
        let ty: Type = parse_quote!(());
        assert_eq!(type_to_gleam(&ty), "Nil");
        let ty: Type = parse_quote!((i32, String, bool));
        assert_eq!(type_to_gleam(&ty), "#(Int, String, Bool)");
    }

    #[test]
    fn type_to_gleam_references() {
        let ty: Type = parse_quote!(&str);
        assert_eq!(type_to_gleam(&ty), "String");
        let ty: Type = parse_quote!(&String);
        assert_eq!(type_to_gleam(&ty), "String");
        let ty: Type = parse_quote!(&mut Vec<i32>);
        assert_eq!(type_to_gleam(&ty), "List(Int)");
    }

    #[test]
    fn type_to_gleam_with_lifetimes() {
        let ty: Type = parse_quote!(Binary<'a>);
        assert_eq!(type_to_gleam(&ty), "BitArray");
        let ty: Type = parse_quote!(Foo<'a, T>);
        assert_eq!(type_to_gleam(&ty), "Foo(T)");
        let ty: Type = parse_quote!(&'a str);
        assert_eq!(type_to_gleam(&ty), "String");
        let ty: Type = parse_quote!(Bar<T, 'env, U>);
        assert_eq!(type_to_gleam(&ty), "Bar(T, U)");
    }

    #[test]
    fn type_to_gleam_slice() {
        let ty: Type = parse_quote!(&[i32]);
        assert_eq!(type_to_gleam(&ty), "List(Int)");
        let ty: Type = parse_quote!(&[u8]);
        assert_eq!(type_to_gleam(&ty), "List(Int)");
    }

    #[test]
    fn type_to_gleam_array() {
        let ty: Type = parse_quote!([u8; 4]);
        assert_eq!(type_to_gleam(&ty), "List(Int)");
        let ty: Type = parse_quote!([String; 10]);
        assert_eq!(type_to_gleam(&ty), "List(String)");
    }

    #[test]
    fn parse_full_pipeline_complex() {
        let src = r#"
    #[gleam_nif]
    pub fn complex(
        env: Env,
        items: Vec<Option<i64>>,
        data: Binary,
        config: HashMap<String, bool>,
    ) -> Result<(i64, String), String> {
        todo!()
    }
    "#;
        let funcs = parse_nif_functions(src);
        assert_eq!(funcs.len(), 1);
        let f = &funcs[0];
        assert_eq!(f.arity, 3);
        assert_eq!(
            f.args[0],
            ("items".into(), "List(option.Option(Int))".into())
        );
        assert_eq!(f.args[1], ("data".into(), "BitArray".into()));
        assert_eq!(
            f.args[2],
            ("config".into(), "dict.Dict(String, Bool)".into())
        );
        assert_eq!(f.ret, "Result(#(Int, String), String)");
    }

    #[test]
    fn parse_nif_with_slice_accepted() {
        let src = r#"
    #[gleam_nif]
    pub fn bad(data: &[u8]) -> i64 {
        0
    }
    "#;
        let funcs = parse_nif_functions(src);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].args[0].1, "List(Int)");
    }

    #[test]
    fn type_to_gleam_nested() {
        let ty: Type = parse_quote!(Vec<Option<i32>>);
        assert_eq!(type_to_gleam(&ty), "List(option.Option(Int))");
        let ty: Type = parse_quote!(Result<Vec<String>, i32>);
        assert_eq!(type_to_gleam(&ty), "Result(List(String), Int)");
        let ty: Type = parse_quote!(Option<Result<i32, String>>);
        assert_eq!(type_to_gleam(&ty), "option.Option(Result(Int, String))");
    }

    #[test]
    fn type_to_gleam_resource_arc() {
        let ty: Type = parse_quote!(ResourceArc<MyStruct>);
        assert_eq!(type_to_gleam(&ty), "Resource");
    }

    #[test]
    fn type_to_gleam_custom_type_no_generics() {
        let ty: Type = parse_quote!(MyCustomType);
        assert_eq!(type_to_gleam(&ty), "MyCustomType");
    }

    #[test]
    fn type_to_gleam_full_path() {
        let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
        assert_eq!(type_to_gleam(&ty), "dict.Dict(String, Int)");
        let ty: Type = parse_quote!(std::string::String);
        assert_eq!(type_to_gleam(&ty), "String");
    }
}
