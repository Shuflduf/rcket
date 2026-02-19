extern crate proc_macro;
use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, LitStr, Type, Variant,
    parse_macro_input,
};

#[proc_macro_derive(Node, attributes(token, regex))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let (output_type, parse_body) = match &input.data {
        Data::Struct(s) => derive_struct(s),
        Data::Enum(e) => derive_enum(e),
        _ => (quote! { Self }, quote! { todo!() }),
    };

    quote! {
        impl Node for #name {
            type Output = #output_type;
            fn parse(input: &str) -> Option<Self::Output> {
                #parse_body
            }
        }
    }
    .into()
}

fn derive_struct(s: &DataStruct) -> (TokenStream2, TokenStream2) {
    let types: Vec<&Type> = match &s.fields {
        Fields::Unnamed(f) => f.unnamed.iter().map(|f| &f.ty).collect(),
        Fields::Named(f) => f.named.iter().map(|f| &f.ty).collect(),
        Fields::Unit => vec![],
    };
    let output_type = match types.len() {
        0 => quote! { () },
        1 => quote! { #(#types)* },
        _ => quote! { (#(#types),*) },
    };
    (output_type, quote! { todo!() })
}

fn derive_enum(e: &DataEnum) -> (TokenStream2, TokenStream2) {
    let arms: Vec<TokenStream2> = e.variants.iter().flat_map(variant_arms).collect();
    (quote! { Self }, quote! { #(#arms)* None })
}

fn variant_arms(variant: &Variant) -> Vec<TokenStream2> {
    let name = &variant.ident;
    variant
        .attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("token") {
                let lit = attr.parse_args::<LitStr>().ok()?;
                token_arm(name, &lit)
            } else if attr.path().is_ident("regex") {
                let lit = attr.parse_args::<LitStr>().ok()?;
                let ty = single_unnamed_field(variant)?;
                Some(regex_arm(name, &lit, ty))
            } else {
                None
            }
        })
        .collect()
}

fn token_arm(name: &Ident, lit: &LitStr) -> Option<TokenStream2> {
    Some(quote! {
        if input == #lit {
            return Some(Self::#name);
        }
    })
}

fn regex_arm(name: &Ident, lit: &LitStr, ty: &Type) -> TokenStream2 {
    quote! {
        {
            let re = ::regex::Regex::new(#lit).unwrap();
            if let Some(m) = re.find(input) {
                if m.start() == 0 && m.end() == input.len() {
                    if let Ok(val) = input.parse::<#ty>() {
                        return Some(Self::#name(val));
                    }
                }
            }
        }
    }
}

fn single_unnamed_field(variant: &Variant) -> Option<&Type> {
    if let Fields::Unnamed(fields) = &variant.fields {
        if fields.unnamed.len() == 1 {
            return Some(&fields.unnamed[0].ty);
        }
    }
    None
}
