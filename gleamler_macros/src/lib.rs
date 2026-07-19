use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat};

#[proc_macro_attribute]
pub fn gleam_nif(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let ffi_fn_name = syn::Ident::new(&format!("ffi_{}", fn_name), fn_name.span());

    let mut args_decoding = Vec::new();
    let mut args_names = Vec::new();

    for (i, arg) in input_fn.sig.inputs.iter().enumerate() {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let arg_ident = &pat_ident.ident;
                let arg_type = &pat_type.ty;

                args_names.push(arg_ident.clone());

                args_decoding.push(quote! {
                    let #arg_ident: #arg_type = <#arg_type as crate::Decoder>::decode(*argv.add(#i));
                });
            }
        }
    }

    let expanded = quote! {
        #input_fn

        pub unsafe extern "C" fn #ffi_fn_name(
            _env: *mut crate::ErlNifEnv,
            _argc: i32,
            argv: *const crate::ErlNifTerm,
        ) -> crate::ErlNifTerm {
            unsafe {
                #(#args_decoding)*

                let result = #fn_name(#(#args_names),*);
                
                <i64 as crate::Encoder>::encode(result)
            }
        }
    };

    TokenStream::from(expanded)
}