use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, GenericArgument, Ident, Index, Path,
    PathArguments, Type, Variant, parse_macro_input,
};

pub(crate) fn derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = &input.ident;

    let token_type = input
        .attrs
        .iter()
        .find(|attribute| attribute.path().is_ident("node"))
        .and_then(|attribute| {
            attribute
                .parse_args_with(|stream: syn::parse::ParseStream| {
                    let _key: Ident = stream.parse()?;
                    let _equals: syn::Token![=] = stream.parse()?;
                    let type_ident: Ident = stream.parse()?;
                    Ok(type_ident)
                })
                .ok()
        })
        .unwrap_or_else(|| Ident::new("Token", proc_macro2::Span::call_site()));

    let (output_type, parse_body) = match &input.data {
        Data::Struct(data_struct) => derive_struct(data_struct, type_name, &token_type),
        Data::Enum(data_enum) => derive_enum(data_enum, &token_type),
        _ => (quote! { Self }, quote! { todo!() }),
    };

    let display_impl = match &input.data {
        Data::Struct(data_struct) => display_impl_struct(data_struct, type_name),
        Data::Enum(data_enum) => display_impl_enum(data_enum, type_name),
        _ => quote! {},
    };

    quote! {
        impl ::rcket::Node for #type_name {
            type Token = #token_type;
            type Output = #output_type;
            fn parse_one(tokens: &[#token_type]) -> Option<(Self::Output, &[#token_type])> {
                #parse_body
            }
        }
        #display_impl
    }
    .into()
}

fn derive_struct(
    data_struct: &DataStruct,
    type_name: &Ident,
    token_type: &Ident,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let fields: Vec<&syn::Field> = match &data_struct.fields {
        Fields::Unnamed(fields) => fields.unnamed.iter().collect(),
        Fields::Named(fields) => fields.named.iter().collect(),
        Fields::Unit => vec![],
    };

    let mut parse_steps = vec![];
    let mut field_bindings: Vec<Ident> = vec![];

    for (field_index, field) in fields.iter().enumerate() {
        let binding = format_ident!("field_{}", field_index);
        field_bindings.push(binding.clone());

        let token_attribute = field.attrs.iter().find(|attribute| attribute.path().is_ident("token"));

        if let Some(token_attribute) = token_attribute {
            let path = token_attribute.parse_args::<Path>().unwrap();
            let first_segment_ident = &path.segments[0].ident;
            let token_pattern = if first_segment_ident == "Symbol" {
                quote! { #token_type::Symbol(#path) }
            } else {
                quote! { #token_type::Keyword(#path) }
            };
            parse_steps.push(quote! {
                let tokens = if let Some((#token_pattern, rest)) = tokens.split_first() { rest } else { return None; };
                let #binding = ();
            });
        } else if let Some(inner_type) = unwrap_box(&field.ty) {
            parse_steps.push(quote! {
                let (#binding, tokens) = <#inner_type as ::rcket::Node>::parse_one(tokens)?;
                let #binding = ::std::boxed::Box::new(#binding);
            });
        } else {
            let field_type = &field.ty;
            parse_steps.push(quote! {
                let (#binding, tokens) = <#field_type as ::rcket::Node>::parse_one(tokens)?;
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

fn derive_enum(
    data_enum: &DataEnum,
    token_type: &Ident,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let variant_match_arms: Vec<proc_macro2::TokenStream> = data_enum
        .variants
        .iter()
        .flat_map(|variant| variant_arms(variant, token_type))
        .collect();
    (quote! { Self }, quote! { #(#variant_match_arms)* None })
}

fn variant_arms(variant: &Variant, token_type: &Ident) -> Vec<proc_macro2::TokenStream> {
    let variant_name = &variant.ident;

    let attribute_arms: Vec<proc_macro2::TokenStream> = variant
        .attrs
        .iter()
        .filter_map(|attribute| {
            if attribute.path().is_ident("token") {
                let path = attribute.parse_args::<Path>().ok()?;
                Some(token_arm(variant_name, &path, token_type))
            } else if attribute.path().is_ident("extract") {
                let path = attribute.parse_args::<Path>().ok()?;
                Some(extract_arm(variant_name, &path, token_type))
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

fn token_arm(variant_name: &Ident, path: &Path, token_type: &Ident) -> proc_macro2::TokenStream {
    let first_segment_ident = &path.segments[0].ident;
    let token_pattern = if first_segment_ident == "Symbol" {
        quote! { #token_type::Symbol(#path) }
    } else {
        quote! { #token_type::Keyword(#path) }
    };
    quote! {
        if let Some((#token_pattern, rest)) = tokens.split_first() {
            return Some((Self::#variant_name, rest));
        }
    }
}

fn extract_arm(variant_name: &Ident, path: &Path, token_type: &Ident) -> proc_macro2::TokenStream {
    quote! {
        if let Some((#token_type::Literal(#path(value)), rest)) = tokens.split_first() {
            return Some((Self::#variant_name(value.clone()), rest));
        }
    }
}

fn bare_arm(variant_name: &Ident, inner_type: &Type) -> proc_macro2::TokenStream {
    quote! {
        if let Some((result, rest)) = <#inner_type as ::rcket::Node>::parse_one(tokens) {
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
            if let PathArguments::AngleBracketed(angle_arguments) = &segments[0].arguments {
                if angle_arguments.args.len() == 1 {
                    if let GenericArgument::Type(inner_type) = &angle_arguments.args[0] {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}

fn display_impl_struct(data_struct: &DataStruct, type_name: &Ident) -> proc_macro2::TokenStream {
    let type_name_str = type_name.to_string();

    let field_writes: Vec<proc_macro2::TokenStream> = match &data_struct.fields {
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .filter(|(_, field)| !field.attrs.iter().any(|attribute| attribute.path().is_ident("token")))
            .map(|(field_index, _)| {
                let index = Index::from(field_index);
                quote! { write!(formatter, " {}", self.#index)?; }
            })
            .collect(),
        Fields::Named(fields) => fields
            .named
            .iter()
            .filter(|field| !field.attrs.iter().any(|attribute| attribute.path().is_ident("token")))
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                quote! { write!(formatter, " {}", self.#field_name)?; }
            })
            .collect(),
        Fields::Unit => vec![],
    };

    quote! {
        impl ::std::fmt::Display for #type_name {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(formatter, "({}",  #type_name_str)?;
                #(#field_writes)*
                write!(formatter, ")")
            }
        }
    }
}

fn display_impl_enum(data_enum: &DataEnum, type_name: &Ident) -> proc_macro2::TokenStream {
    let match_arms: Vec<proc_macro2::TokenStream> = data_enum
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_name_str = variant_name.to_string();

            let has_token = variant.attrs.iter().any(|attribute| attribute.path().is_ident("token"));
            let has_extract = variant.attrs.iter().any(|attribute| attribute.path().is_ident("extract"));

            if has_token {
                quote! { Self::#variant_name => write!(formatter, #variant_name_str), }
            } else if has_extract {
                quote! { Self::#variant_name(value) => write!(formatter, "({} {})", #variant_name_str, value), }
            } else if single_unnamed_field(variant).is_some() {
                quote! { Self::#variant_name(inner) => ::std::fmt::Display::fmt(inner, formatter), }
            } else {
                quote! { Self::#variant_name => write!(formatter, #variant_name_str), }
            }
        })
        .collect();

    quote! {
        impl ::std::fmt::Display for #type_name {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}
