use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse_quote;

use super::context::Context;

pub(crate) fn decoder(ctx: &Context, inner: TokenStream) -> TokenStream {
    let ident = ctx.ident;
    let generics = ctx.generics;
    let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

    let mut impl_generics = generics.clone();
    let decode_lifetime = syn::Lifetime::new("'__gleamler_decode_lifetime", Span::call_site());
    let lifetime_def = syn::LifetimeParam::new(decode_lifetime.clone());
    impl_generics
        .params
        .push(syn::GenericParam::Lifetime(lifetime_def));

    let where_clause = impl_generics.make_where_clause();

    for lifetime in ctx.lifetimes.iter() {
        let bound: syn::WherePredicate = parse_quote! {
            '__gleamler_decode_lifetime: #lifetime
        };
        where_clause.predicates.push(bound);
    }
    for type_parameter in ctx.type_parameters.iter() {
        let ty = &type_parameter.ident;
        let bound: syn::WherePredicate = parse_quote! {
            #ty: ::gleamler::Decoder<'__gleamler_decode_lifetime>
        };
        where_clause.predicates.push(bound);
    }

    let (impl_generics, _, where_clause) = impl_generics.split_for_impl();

    quote! {
        impl #impl_generics ::gleamler::Decoder<'__gleamler_decode_lifetime> for #ident #ty_generics #where_clause {
            #[allow(clippy::needless_borrow)]
            fn decode(term: ::gleamler::Term<'__gleamler_decode_lifetime>) -> ::gleamler::NifResult<Self> {
                #inner
            }
        }
    }
}

pub(crate) fn encoder(ctx: &Context, inner: TokenStream) -> TokenStream {
    let ident = ctx.ident;
    let mut generics = ctx.generics.clone();

    let where_clause = generics.make_where_clause();

    for type_parameter in ctx.type_parameters.iter() {
        let ty = &type_parameter.ident;
        let bound: syn::WherePredicate = parse_quote! {
            #ty: ::gleamler::Encoder
        };
        where_clause.predicates.push(bound);
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::gleamler::Encoder for #ident #ty_generics #where_clause {
            #[allow(clippy::needless_borrow)]
            fn encode<'__gleamler__encode_lifetime>(&self, env: ::gleamler::Env<'__gleamler__encode_lifetime>) -> ::gleamler::Term<'__gleamler__encode_lifetime> {
                #inner
            }
        }
    }
}
