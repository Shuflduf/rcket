extern crate proc_macro;
use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, GenericArgument, Ident, Path,
    PathArguments, Type, Variant, parse_macro_input,
};

mod lex;

#[proc_macro_derive(Node, attributes(token, extract))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = &input.ident;

    let (output_type, parse_body) = match &input.data {
        Data::Struct(data_struct) => derive_struct(data_struct, type_name),
        Data::Enum(data_enum) => derive_enum(data_enum),
        _ => (quote! { Self }, quote! { todo!() }),
    };

    quote! {
        impl ::rcket::Node for #type_name {
            type Token = Token;
            type Output = #output_type;
            fn parse(tokens: &[Token]) -> Option<(Self::Output, &[Token])> {
                #parse_body
            }
        }
    }
    .into()
}

fn derive_struct(
    data_struct: &DataStruct,
    type_name: &Ident,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let field_types: Vec<&Type> = match &data_struct.fields {
        Fields::Unnamed(fields) => fields.unnamed.iter().map(|field| &field.ty).collect(),
        Fields::Named(fields) => fields.named.iter().map(|field| &field.ty).collect(),
        Fields::Unit => vec![],
    };

    let mut parse_steps = vec![];
    let mut field_bindings: Vec<Ident> = vec![];

    for (field_index, field_type) in field_types.iter().enumerate() {
        let binding = format_ident!("field_{}", field_index);
        field_bindings.push(binding.clone());

        if let Some(inner_type) = unwrap_box(field_type) {
            parse_steps.push(quote! {
                let (#binding, tokens) = <#inner_type as ::rcket::Node>::parse(tokens)?;
                let #binding = ::std::boxed::Box::new(#binding);
            });
        } else {
            parse_steps.push(quote! {
                let (#binding, tokens) = <#field_type as ::rcket::Node>::parse(tokens)?;
            });
        }
    }

    let parse_body = quote! {
        (|| -> Option<_> {
            #(#parse_steps)*
            Some((#type_name(#(#field_bindings),*), tokens))
        })()
    };

    (quote! { Self }, parse_body)
}

fn derive_enum(data_enum: &DataEnum) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let variant_match_arms: Vec<proc_macro2::TokenStream> =
        data_enum.variants.iter().flat_map(variant_arms).collect();
    (quote! { Self }, quote! { #(#variant_match_arms)* None })
}

fn variant_arms(variant: &Variant) -> Vec<proc_macro2::TokenStream> {
    let variant_name = &variant.ident;

    let attribute_arms: Vec<proc_macro2::TokenStream> = variant
        .attrs
        .iter()
        .filter_map(|attribute| {
            if attribute.path().is_ident("token") {
                let path = attribute.parse_args::<Path>().ok()?;
                Some(token_arm(variant_name, &path))
            } else if attribute.path().is_ident("extract") {
                let path = attribute.parse_args::<Path>().ok()?;
                Some(extract_arm(variant_name, &path))
            } else {
                None
            }
        })
        .collect();

    if !attribute_arms.is_empty() {
        return attribute_arms;
    }

    if let Some(inner_type) = single_unnamed_field(variant) {
        vec![bare_arm(variant_name, inner_type)]
    } else {
        vec![]
    }
}

fn token_arm(variant_name: &Ident, path: &Path) -> proc_macro2::TokenStream {
    let first_segment_ident = &path.segments[0].ident;
    let token_pattern = if first_segment_ident == "Symbol" {
        quote! { Token::Symbol(#path) }
    } else {
        quote! { Token::Keyword(#path) }
    };
    quote! {
        if let Some((#token_pattern, rest)) = tokens.split_first() {
            return Some((Self::#variant_name, rest));
        }
    }
}

fn extract_arm(variant_name: &Ident, path: &Path) -> proc_macro2::TokenStream {
    quote! {
        if let Some((Token::Literal(#path(value)), rest)) = tokens.split_first() {
            return Some((Self::#variant_name(value.clone()), rest));
        }
    }
}

fn bare_arm(variant_name: &Ident, inner_type: &Type) -> proc_macro2::TokenStream {
    quote! {
        if let Some((result, rest)) = <#inner_type as ::rcket::Node>::parse(tokens) {
            return Some((Self::#variant_name(result), rest));
        }
    }
}

pub(crate) fn single_unnamed_field(variant: &Variant) -> Option<&Type> {
    if let Fields::Unnamed(fields) = &variant.fields {
        if fields.unnamed.len() == 1 {
            return Some(&fields.unnamed[0].ty);
        }
    }
    None
}

fn unwrap_box(field_type: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = field_type {
        let segments = &type_path.path.segments;
        if segments.len() == 1 && segments[0].ident == "Box" {
            if let PathArguments::AngleBracketed(angle_args) = &segments[0].arguments {
                if angle_args.args.len() == 1 {
                    if let GenericArgument::Type(inner_type) = &angle_args.args[0] {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_derive(Lex, attributes(token, regex, seq, choice))]
pub fn derive_lex(input: TokenStream) -> TokenStream {
    lex::derive_lex(input)
}
