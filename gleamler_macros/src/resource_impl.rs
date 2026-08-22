use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashSet;
use syn::{meta::ParseNestedMeta, LitBool, LitStr};

pub struct Attributes {
    register: bool,
    name: Option<String>,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            register: true,
            name: None,
        }
    }
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> syn::parse::Result<()> {
        if meta.path.is_ident("register") {
            let value: LitBool = meta.value()?.parse()?;
            self.register = value.value;
            Ok(())
        } else if meta.path.is_ident("name") {
            let value: LitStr = meta.value()?.parse()?;
            self.name = Some(value.value());
            Ok(())
        } else {
            Err(meta.error("Unsupported macro attribute. Expecting register or name."))
        }
    }
}

pub fn transcoder_decorator(attrs: Attributes, mut input: syn::ItemImpl) -> TokenStream {
    let type_path = match *input.self_ty {
        syn::Type::Path(ref type_path) => type_path.clone(),
        _ => panic!("Can only implement trait on concrete types"),
    };

    let mut to_add: HashSet<String> = HashSet::new();
    let mut already_has: HashSet<String> = HashSet::new();

    for item in input.items.iter() {
        if let syn::ImplItem::Fn(f) = item {
            to_add.insert(
                format!("IMPLEMENTS_{}", f.sig.ident.to_string().to_uppercase()).to_string(),
            );
        }

        if let syn::ImplItem::Const(f) = item {
            already_has.insert(f.ident.to_string());
        }
    }

    for add in to_add.difference(&already_has) {
        let ident = syn::Ident::new(add, Span::call_site());
        let impl_item: syn::ImplItem = syn::parse_quote!(const #ident: bool = true;);
        input.items.push(impl_item);
    }

    let mut res = quote!(#input);

    if attrs.register {
        let fallback_fn = syn::Ident::new(
            &format!("__gleamler_fallback_reg_{}", type_path.path.segments.last().unwrap().ident),
            Span::call_site(),
        );
        
        let submit = if let Some(name) = attrs.name {
            quote!(
                ::gleamler::codegen_runtime::inventory::submit!(
                    ::gleamler::codegen_runtime::ResourceRegistration::new::<#type_path>().with_name(#name)
                );

                #[::gleamler::codegen_runtime::ctor::ctor]
                fn #fallback_fn() {
                    ::gleamler::resource::Registration::submit_fallback(
                        ::gleamler::resource::Registration::new::<#type_path>().with_name(#name)
                    );
                }
            )
        } else {
            quote!(
                ::gleamler::codegen_runtime::inventory::submit!(
                    ::gleamler::codegen_runtime::ResourceRegistration::new::<#type_path>()
                );

                #[::gleamler::codegen_runtime::ctor::ctor]
                fn #fallback_fn() {
                    ::gleamler::resource::Registration::submit_fallback(
                        ::gleamler::resource::Registration::new::<#type_path>()
                    );
                }
            )
        };
        res.extend(quote!(#submit));
    }

    res
}