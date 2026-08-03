use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatIdent, Ident, Token};
use syn::punctuated::Punctuated;

#[proc_macro_attribute]
pub fn gleam_nif(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let ffi_fn_name = syn::Ident::new(&format!("ffi_{}", fn_name), fn_name.span());
    let nif_const_name = syn::Ident::new(&format!("__GLEAMLER_NIF_{}", fn_name), fn_name.span());

    let mut args_decoding = Vec::new();
    let mut args_names = Vec::new();
    let mut arg_idx: usize = 0;
    let mut has_env = false;

    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                let arg_ident = ident;
                let arg_type = &pat_type.ty;

                let type_str = quote!(#arg_type).to_string().replace(' ', "");
                if type_str.contains("Env") && arg_idx == 0 {
                    has_env = true;
                    args_names.push(arg_ident.clone());
                    args_decoding.push(quote! { let #arg_ident = env; });
                    arg_idx += 1;
                    continue;
                }

                args_names.push(arg_ident.clone());
                args_decoding.push(quote! {
                    let #arg_ident: #arg_type = match args[#arg_idx].decode() {
                        Ok(v) => v,
                        Err(_) => return Err(::gleamler::Error::BadArg),
                    };
                });
                arg_idx += 1;
            }
        }
    }

    let arity = if has_env { arg_idx - 1 } else { arg_idx } as u32;

    let expanded = quote! {
        #input_fn

        pub unsafe extern "C" fn #ffi_fn_name(
            nif_env: ::gleamler::codegen_runtime::NIF_ENV,
            argc: ::gleamler::codegen_runtime::c_int,
            argv: *const ::gleamler::codegen_runtime::NIF_TERM,
        ) -> ::gleamler::codegen_runtime::NIF_TERM {
            eprintln!("[Rust ffi] {} called, argc={}", stringify!(#fn_name), argc);
            use ::gleamler::codegen_runtime::NifReturnable;

            let lifetime = ();
            let env = unsafe { ::gleamler::Env::new(&lifetime, nif_env) };

            let terms = unsafe {
                std::slice::from_raw_parts(argv, argc as usize)
                    .iter()
                    .map(|term| ::gleamler::Term::new(env, *term))
                    .collect::<Vec<::gleamler::Term>>()
            };
            let args: &[::gleamler::Term] = &terms;

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
        pub const #nif_const_name: ::gleamler::nif::Nif = ::gleamler::nif::Nif {
            name: concat!(stringify!(#fn_name), "\0").as_ptr()
                as *const ::gleamler::codegen_runtime::c_char,
            arity: #arity,
            flags: 0,
            raw_func: #ffi_fn_name,
        };
    };

    TokenStream::from(expanded)
}

struct InitInput {
    names: Punctuated<Ident, Token![,]>,
}

impl syn::parse::Parse for InitInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);
        let names = content.parse_terminated(Ident::parse, Token![,])?;
        Ok(Self { names })
    }
}

#[proc_macro]
pub fn init_nifs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as InitInput);
    
    let nif_consts: Vec<_> = input.names.iter().map(|name| {
        syn::Ident::new(&format!("__GLEAMLER_NIF_{}", name), name.span())
    }).collect();
    
    let funcs: Vec<_> = nif_consts.iter().map(|const_name| {
        quote! {
            ::gleamler::sys::ErlNifFunc {
                name: #const_name.name,
                arity: #const_name.arity,
                flags: #const_name.flags,
                function: #const_name.raw_func,
            }
        }
    }).collect();
    
    let entry_body = quote! {
        use ::gleamler::sys::{ErlNifFunc, ErlNifEntry, NIF_MAJOR_VERSION, NIF_MINOR_VERSION, ERL_NIF_ENTRY_OPTIONS};
        use ::gleamler::codegen_runtime::min_erts;
        use ::gleamler::wrapper::get_nif_resource_type_init_size;
        use std::ffi::c_char;

        let funcs = vec![#(#funcs),*];
        
        let funcs_ptr = funcs.as_ptr();
        let num_of_funcs = funcs.len() as i32;
        std::mem::forget(funcs);

        let entry = Box::new(ErlNifEntry {
            major: NIF_MAJOR_VERSION,
            minor: NIF_MINOR_VERSION,
            name: b"gleamler_nif\0".as_ptr() as *const c_char,
            num_of_funcs,
            funcs: funcs_ptr,
            load: None,
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
    
    let expanded = quote! {
        #[cfg(not(target_os = "windows"))]
        #[unsafe(no_mangle)]
        pub extern "C" fn nif_init() -> *const ::gleamler::sys::ErlNifEntry {
            use std::sync::Once;
            static INIT: Once = Once::new();
            INIT.call_once(|| {
                unsafe { ::gleamler::codegen_runtime::internal_write_symbols() };
            });
            #entry_body
        }

        #[cfg(target_os = "windows")]
        #[unsafe(no_mangle)]
        pub extern "C" fn nif_init(callbacks: *mut ::gleamler::codegen_runtime::DynNifCallbacks) -> *const ::gleamler::sys::ErlNifEntry {
            unsafe {
                ::gleamler::codegen_runtime::internal_set_symbols(*callbacks);
            }
            #entry_body
        }
    };
    
    TokenStream::from(expanded)
}