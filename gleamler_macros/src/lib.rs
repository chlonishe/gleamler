#![recursion_limit = "128"]

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{FnArg, Ident, ItemFn, Pat, PatIdent, Token, parse_macro_input};

mod context;
mod encode_decode_templates;
mod map;
mod record;
mod resource_impl;
mod tagged_enum;
mod tuple;
mod unit_enum;
mod untagged_enum;

#[derive(Debug)]
enum GleamlerAttr {
    Encode,
    Decode,
    Tag(String),
}

#[proc_macro_attribute]
pub fn gleam_nif(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let ffi_fn_name = syn::Ident::new(&format!("ffi_{}", fn_name), fn_name.span());
    let nif_const_name = syn::Ident::new(&format!("__GLEAMLER_NIF_{}", fn_name), fn_name.span());

    let mut nif_flags = quote!(0);
    let mut alias: Option<syn::LitStr> = None;

    if !attr.is_empty() {
        use syn::parse::Parser;
        let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
        let metas = parser.parse(attr)
            .expect("gleam_nif attributes must be comma-separated meta items, e.g. dirty_cpu, alias = \"name\"");
        for meta in metas {
            if meta.path().is_ident("dirty_cpu") {
                nif_flags = quote!(
                    ::gleamler::schedule::SchedulerFlags::DirtyCpu
                        as ::gleamler::codegen_runtime::c_uint
                );
            } else if meta.path().is_ident("dirty_io") {
                nif_flags = quote!(
                    ::gleamler::schedule::SchedulerFlags::DirtyIo
                        as ::gleamler::codegen_runtime::c_uint
                );
            } else if meta.path().is_ident("alias") {
                let expr: syn::Expr = meta
                    .require_name_value()
                    .expect("alias must be a name-value pair: alias = \"name\"")
                    .value
                    .clone();
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = expr
                {
                    alias = Some(lit_str);
                } else {
                    panic!("alias value must be a string literal");
                }
            } else {
                panic!("unknown gleam_nif attribute");
            }
        }
    }

    let export_name = alias
        .as_ref()
        .map(|s| s.value())
        .unwrap_or_else(|| fn_name.to_string());
    let export_name_lit = syn::LitStr::new(&format!("{}\0", export_name), fn_name.span());
    let mut args_decoding = Vec::new();
    let mut args_names = Vec::new();
    let mut nif_arg_idx: usize = 0;

    fn is_reference_to_env(ty: &syn::Type) -> bool {
        match ty {
            syn::Type::Reference(type_ref) => is_env_type(&type_ref.elem),
            _ => false,
        }
    }

    fn is_env_type(ty: &syn::Type) -> bool {
        match ty {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident == "Env")
                .unwrap_or(false),
            syn::Type::Reference(type_ref) => is_env_type(&type_ref.elem),
            _ => false,
        }
    }

    for arg in &input_fn.sig.inputs {
        let pat_type = match arg {
            FnArg::Typed(pat_type) => pat_type,
            FnArg::Receiver(rec) => {
                return syn::Error::new_spanned(
                    rec,
                    "gleam_nif: methods with a `self` receiver are not supported",
                )
                .to_compile_error()
                .into();
            }
        };
        let ident = match &*pat_type.pat {
            Pat::Ident(PatIdent { ident, .. }) => ident,
            other => {
                return syn::Error::new_spanned(
                    other,
                    "gleam_nif: argument patterns are not supported; \
                     give the argument a plain name and destructure it in the function body",
                )
                .to_compile_error()
                .into();
            }
        };

        let arg_ident = ident;
        let arg_type = &pat_type.ty;

        let is_env = is_env_type(&pat_type.ty);

        if is_reference_to_env(&pat_type.ty) {
            return syn::Error::new_spanned(
                pat_type,
                "gleam_nif: `&Env` and `&mut Env` arguments are not supported; \
                 pass `Env` by value instead (it implements `Copy`)",
            )
            .to_compile_error()
            .into();
        }

        if is_env {
            args_names.push(arg_ident.clone());
            args_decoding.push(quote! { let #arg_ident = env; });
            continue;
        }

        args_names.push(arg_ident.clone());
        args_decoding.push(quote! {
            let #arg_ident: #arg_type = match args[#nif_arg_idx].decode() {
                Ok(v) => v,
                Err(_) => return Err(::gleamler::Error::BadArg),
            };
        });
        nif_arg_idx += 1;
    }

    let arity = nif_arg_idx as u32;

    let expanded = quote! {
        #input_fn

        unsafe extern "C" fn #ffi_fn_name(
            nif_env: ::gleamler::codegen_runtime::NIF_ENV,
            argc: ::gleamler::codegen_runtime::c_int,
            argv: *const ::gleamler::codegen_runtime::NIF_TERM,
        ) -> ::gleamler::codegen_runtime::NIF_TERM {
            use ::gleamler::codegen_runtime::NifReturnable;

            unsafe {
                ::gleamler::codegen_runtime::set_nif_continuation(
                    #export_name_lit.as_ptr() as *const _,
                    #ffi_fn_name,
                    argc,
                    argv,
                );
            }

            let lifetime = ();
            let env = unsafe { ::gleamler::Env::new(&lifetime, nif_env) };

            let terms: Vec<::gleamler::Term> = if argc == 0 {
                Vec::new()
            } else {
                unsafe {
                    std::slice::from_raw_parts(argv, argc as usize)
                        .iter()
                        .map(|term| ::gleamler::Term::new(env, *term))
                        .collect()
                }
            };
            let args: &[::gleamler::Term] = &terms;

            if (argc as usize) != (#arity as usize) {
                return unsafe { ::gleamler::codegen_runtime::NifReturned::BadArg.apply(env) };
            }
            let result: std::thread::Result<Result<_, ::gleamler::Error>> =
                std::panic::catch_unwind(move || {
                    #(#args_decoding)*
                    Ok(#fn_name(#(#args_names),*))
                });

            let nif_returned = ::gleamler::codegen_runtime::handle_nif_result(result, env);
            unsafe { nif_returned.apply(env) }
        }

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        pub const #nif_const_name: ::gleamler::Nif = ::gleamler::Nif {
            name: #export_name_lit.as_ptr()
                as *const ::gleamler::codegen_runtime::c_char,
            arity: #arity,
            flags: #nif_flags,
            raw_func: #ffi_fn_name,
        };
    };

    TokenStream::from(expanded)
}

struct InitInput {
    module: Option<syn::LitStr>,
    load: Option<syn::ExprPath>,
    names: Punctuated<Ident, Token![,]>,
}

impl syn::parse::Parse for InitInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut module = None;
        let mut load = None;

        if input.peek(syn::LitStr) {
            module = Some(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        if input.peek(Ident) && input.peek2(Token![=]) {
            let key: Ident = input.parse()?;
            if key != "load" {
                return Err(syn::Error::new(key.span(), "expected `load`"));
            }
            input.parse::<Token![=]>()?;
            load = Some(input.parse::<syn::ExprPath>()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let names = if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            content.parse_terminated(Ident::parse, Token![,])?
        } else {
            Punctuated::new()
        };

        Ok(Self {
            module,
            load,
            names,
        })
    }
}

#[proc_macro]
pub fn init_nifs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as InitInput);

    let (funcs_static, num_of_funcs) = if input.names.is_empty() {
        (
            quote! {
                let funcs: &'static [::gleamler::sys::ErlNifFunc] =
                    ::gleamler::nifs::__generated_registry::NIFS;
                let funcs_ptr = funcs.as_ptr();
                let num_of_funcs = funcs.len() as i32;
            },
            quote! { num_of_funcs },
        )
    } else {
        let nif_consts: Vec<_> = input
            .names
            .iter()
            .map(|name| syn::Ident::new(&format!("__GLEAMLER_NIF_{}", name), name.span()))
            .collect();

        let funcs: Vec<_> = nif_consts
            .iter()
            .map(|const_name| {
                quote! {
                    ::gleamler::sys::ErlNifFunc {
                        name: #const_name.name,
                        arity: #const_name.arity,
                        flags: #const_name.flags,
                        function: #const_name.raw_func,
                    }
                }
            })
            .collect();

        (
            quote! {
                let funcs: &'static [_] = Box::leak(vec![#(#funcs),*].into_boxed_slice());
                let funcs_ptr = funcs.as_ptr();
                let num_of_funcs = funcs.len() as i32;
            },
            quote! { num_of_funcs },
        )
    };

    let module_name = input
        .module
        .as_ref()
        .map(|m| format!("{}\0", m.value()))
        .unwrap_or_else(|| "gleamler_nif_ffi\0".to_string());
    let module_name_lit = syn::LitStr::new(&module_name, proc_macro2::Span::call_site());

    let load_body = if let Some(load_path) = &input.load {
        quote! {
            let env = ::gleamler::Env::new_init_env(&(), env);
            let load_info = ::gleamler::Term::new(env, load_info);
            if !::gleamler::resource::Registration::register_all_collected(env).is_ok() {
                return 1;
            }
            ::gleamler::codegen_runtime::handle_nif_init_call(#load_path, env, load_info)
        }
    } else {
        quote! {
            let env = ::gleamler::Env::new_init_env(&(), env);
            let load_info = ::gleamler::Term::new(env, load_info);
            ::gleamler::codegen_runtime::handle_nif_init_call(
                |env, _info| {
                    ::gleamler::resource::Registration::register_all_collected(env).is_ok()
                },
                env,
                load_info,
            )
        }
    };

    let entry_body = quote! {
        use ::gleamler::sys::{ErlNifFunc, ErlNifEntry, NIF_MAJOR_VERSION, NIF_MINOR_VERSION, ERL_NIF_ENTRY_OPTIONS};
        use ::gleamler::codegen_runtime::min_erts;
        use ::gleamler::wrapper::get_nif_resource_type_init_size;
        use std::ffi::c_char;

        #funcs_static

        let entry = Box::new(ErlNifEntry {
            major: NIF_MAJOR_VERSION,
            minor: NIF_MINOR_VERSION,
            name: #module_name_lit.as_ptr() as *const c_char,
            num_of_funcs: #num_of_funcs,
            funcs: funcs_ptr,
            load: Some({
                unsafe extern "C" fn __gleamler_nif_load(
                    env: *mut ::gleamler::sys::ErlNifEnv,
                    _priv_data: *mut *mut ::gleamler::sys::c_void,
                    load_info: ::gleamler::sys::ERL_NIF_TERM,
                ) -> ::gleamler::sys::c_int {
                    unsafe {
                        #load_body
                    }
                }
                __gleamler_nif_load
            }),
            reload: None,
            upgrade: None,
            unload: None,
            vm_variant: b"beam.vanilla\0".as_ptr() as *const c_char,
            options: ERL_NIF_ENTRY_OPTIONS,
            sizeof_ErlNifResourceTypeInit: get_nif_resource_type_init_size(),
            min_erts: min_erts().as_ptr() as *const c_char,
        });

        Box::into_raw(entry) as *const _
    };

    let init_fn_name = {
        let crate_name = std::env::var("CARGO_CRATE_NAME").expect("CARGO_CRATE_NAME is not set");
        syn::Ident::new(
            &format!("{crate_name}_nif_init"),
            proc_macro2::Span::call_site(),
        )
    };
    let primary = std::env::var("GLEAMLER_PRIMARY_NIF_INIT").is_ok()
        || std::env::var("CARGO_PRIMARY_PACKAGE").is_ok();
    let maybe_primary = if primary {
        quote! {
            #[cfg(not(target_os = "windows"))]
            #[unsafe(no_mangle)]
            pub extern "C" fn nif_init() -> *const ::gleamler::sys::ErlNifEntry {
                #init_fn_name()
            }

            #[cfg(target_os = "windows")]
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn nif_init(
                callbacks: *mut ::gleamler::codegen_runtime::DynNifCallbacks,
            ) -> *const ::gleamler::sys::ErlNifEntry {
                unsafe { #init_fn_name(callbacks) }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #[cfg(not(target_os = "windows"))]
        #[unsafe(no_mangle)]
        pub extern "C" fn #init_fn_name() -> *const ::gleamler::sys::ErlNifEntry {
            use std::sync::Once;
            static INIT: Once = Once::new();
            INIT.call_once(|| {
                ::gleamler::codegen_runtime::internal_write_symbols();
            });
            #entry_body
        }

        #[cfg(target_os = "windows")]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn #init_fn_name(
            callbacks: *mut ::gleamler::codegen_runtime::DynNifCallbacks,
        ) -> *const ::gleamler::sys::ErlNifEntry {
            unsafe {
                ::gleamler::codegen_runtime::internal_set_symbols(*callbacks);
            }
            #entry_body
        }

        #maybe_primary
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(NifMap, attributes(gleamler))]
pub fn nif_map(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    map::transcoder_decorator(&ast).into()
}

#[proc_macro_derive(NifTuple, attributes(gleamler))]
pub fn nif_tuple(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    tuple::transcoder_decorator(&ast).into()
}

#[proc_macro_derive(NifRecord, attributes(tag, gleamler))]
pub fn nif_record(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    record::transcoder_decorator(&ast).into()
}

#[proc_macro_derive(NifUnitEnum, attributes(gleamler))]
pub fn nif_unit_enum(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    unit_enum::transcoder_decorator(&ast).into()
}

#[proc_macro_derive(NifTaggedEnum, attributes(gleamler))]
pub fn nif_tagged_enum(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    tagged_enum::transcoder_decorator(&ast).into()
}

#[proc_macro_derive(NifUntaggedEnum, attributes(gleamler))]
pub fn nif_untagged_enum(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    untagged_enum::transcoder_decorator(&ast).into()
}

#[proc_macro_attribute]
pub fn resource_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut attributes = resource_impl::Attributes::default();

    if !args.is_empty() {
        let parser = syn::meta::parser(|meta| attributes.parse(meta));
        syn::parse_macro_input!(args with parser);
    }
    let input = syn::parse_macro_input!(item as syn::ItemImpl);

    resource_impl::transcoder_decorator(attributes, input).into()
}
