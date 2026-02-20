extern crate proc_macro;
use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, GenericArgument, Ident, LitStr, Path,
    PathArguments, Type, Variant, parse_macro_input, punctuated::Punctuated,
};

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

fn single_unnamed_field(variant: &Variant) -> Option<&Type> {
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

// ===== derive(Lex) =====

struct LexPattern {
    kind: LexPatternKind,
    lit: LitStr,
}

enum LexPatternKind {
    Token,
    Regex,
}

impl syn::parse::Parse for LexPattern {
    fn parse(parse_stream: syn::parse::ParseStream) -> syn::Result<Self> {
        let identifier: Ident = parse_stream.parse()?;
        let content;
        syn::parenthesized!(content in parse_stream);
        let lit: LitStr = content.parse()?;
        let kind = if identifier == "token" {
            LexPatternKind::Token
        } else {
            LexPatternKind::Regex
        };
        Ok(LexPattern { kind, lit })
    }
}

#[proc_macro_derive(Lex, attributes(token, regex, seq, choice))]
pub fn derive_lex(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = &input.ident;

    let lex_one_body = match &input.data {
        Data::Enum(data_enum) => derive_lex_enum(data_enum),
        _ => quote! { todo!() },
    };

    quote! {
        impl ::rcket::Lex for #type_name {
            fn lex_one(input: &str) -> Option<(Self, &str)> {
                #lex_one_body
            }
        }
    }
    .into()
}

fn derive_lex_enum(data_enum: &DataEnum) -> proc_macro2::TokenStream {
    let mut arm_pairs: Vec<(usize, proc_macro2::TokenStream)> = vec![];

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        let has_attributes = variant.attrs.iter().any(|attribute| {
            attribute.path().is_ident("token")
                || attribute.path().is_ident("regex")
                || attribute.path().is_ident("seq")
                || attribute.path().is_ident("choice")
        });

        if has_attributes {
            for attribute in &variant.attrs {
                if attribute.path().is_ident("token") {
                    if let Ok(lit) = attribute.parse_args::<LitStr>() {
                        arm_pairs.push((lit.value().len(), token_lex_arm(variant_name, &lit)));
                    }
                } else if attribute.path().is_ident("regex") {
                    if let Ok(lit) = attribute.parse_args::<LitStr>() {
                        arm_pairs.push((
                            lit.value().len(),
                            regex_lex_arm(variant_name, &lit, single_unnamed_field(variant)),
                        ));
                    }
                } else if attribute.path().is_ident("seq") {
                    if let Ok(patterns) = attribute
                        .parse_args_with(Punctuated::<LexPattern, syn::Token![,]>::parse_terminated)
                    {
                        let patterns_vec: Vec<LexPattern> = patterns.into_iter().collect();
                        arm_pairs.push((
                            0,
                            seq_lex_arm(variant_name, &patterns_vec, single_unnamed_field(variant)),
                        ));
                    }
                } else if attribute.path().is_ident("choice") {
                    if let Ok(patterns) = attribute
                        .parse_args_with(Punctuated::<LexPattern, syn::Token![,]>::parse_terminated)
                    {
                        for pattern in patterns.iter() {
                            let arm = match pattern.kind {
                                LexPatternKind::Token => token_lex_arm(variant_name, &pattern.lit),
                                LexPatternKind::Regex => regex_lex_arm(
                                    variant_name,
                                    &pattern.lit,
                                    single_unnamed_field(variant),
                                ),
                            };
                            arm_pairs.push((pattern.lit.value().len(), arm));
                        }
                    }
                }
            }
        } else if let Some(inner_type) = single_unnamed_field(variant) {
            arm_pairs.push((0, bare_lex_arm(variant_name, inner_type)));
        }
    }

    // stable sort descending so longer patterns are tried first
    arm_pairs.sort_by(|first, second| second.0.cmp(&first.0));

    let arms: Vec<proc_macro2::TokenStream> = arm_pairs.into_iter().map(|(_, arm)| arm).collect();
    quote! { #(#arms)* None }
}

fn token_lex_arm(variant_name: &Ident, lit: &LitStr) -> proc_macro2::TokenStream {
    let is_word_token = lit
        .value()
        .chars()
        .all(|character| character.is_alphabetic() || character == '_');
    if is_word_token {
        quote! {
            if let Some(rest) = input.strip_prefix(#lit) {
                if rest.is_empty() || !rest.starts_with(|character: char| character.is_alphanumeric() || character == '_') {
                    return Some((Self::#variant_name, rest));
                }
            }
        }
    } else {
        quote! {
            if let Some(rest) = input.strip_prefix(#lit) {
                return Some((Self::#variant_name, rest));
            }
        }
    }
}

fn regex_lex_arm(
    variant_name: &Ident,
    lit: &LitStr,
    field_type: Option<&Type>,
) -> proc_macro2::TokenStream {
    let anchored = format!("^(?:{})", lit.value());
    let anchored_lit = LitStr::new(&anchored, lit.span());
    if let Some(field_type) = field_type {
        quote! {
            {
                let re = ::regex::Regex::new(#anchored_lit).unwrap();
                if let Some(match_result) = re.find(input) {
                    if match_result.end() > 0 {
                        if let Ok(value) = input[..match_result.end()].parse::<#field_type>() {
                            return Some((Self::#variant_name(value), &input[match_result.end()..]));
                        }
                    }
                }
            }
        }
    } else {
        quote! {
            {
                let re = ::regex::Regex::new(#anchored_lit).unwrap();
                if let Some(match_result) = re.find(input) {
                    if match_result.end() > 0 {
                        let rest = &input[match_result.end()..];
                        if rest.is_empty() || !rest.starts_with(|character: char| character.is_alphanumeric() || character == '_') {
                            return Some((Self::#variant_name, rest));
                        }
                    }
                }
            }
        }
    }
}

fn seq_lex_arm(
    variant_name: &Ident,
    patterns: &[LexPattern],
    field_type: Option<&Type>,
) -> proc_macro2::TokenStream {
    let mut steps: Vec<proc_macro2::TokenStream> = vec![];
    let mut capture_binding: Option<Ident> = None;

    for (index, pattern) in patterns.iter().enumerate() {
        let lit = &pattern.lit;
        match pattern.kind {
            LexPatternKind::Token => {
                steps.push(quote! { let rest = rest.strip_prefix(#lit)?; });
            }
            LexPatternKind::Regex => {
                let anchored = format!("^(?:{})", lit.value());
                let anchored_lit = LitStr::new(&anchored, lit.span());
                let binding = format_ident!("capture_{}", index);
                capture_binding = Some(binding.clone());
                if let Some(capture_type) = field_type {
                    steps.push(quote! {
                        let re = ::regex::Regex::new(#anchored_lit).unwrap();
                        let match_result = re.find(rest)?;
                        if match_result.start() != 0 { return None; }
                        let #binding: #capture_type = rest[..match_result.end()].parse().ok()?;
                        let rest = &rest[match_result.end()..];
                    });
                } else {
                    steps.push(quote! {
                        let re = ::regex::Regex::new(#anchored_lit).unwrap();
                        let match_result = re.find(rest)?;
                        if match_result.start() != 0 { return None; }
                        let rest = &rest[match_result.end()..];
                    });
                }
            }
        }
    }

    let return_value = if let Some(binding) = capture_binding {
        quote! { Self::#variant_name(#binding) }
    } else {
        quote! { Self::#variant_name }
    };

    quote! {
        if let Some(result) = (|| -> Option<(Self, &str)> {
            let rest = input;
            #(#steps)*
            Some((#return_value, rest))
        })() {
            return Some(result);
        }
    }
}

fn bare_lex_arm(variant_name: &Ident, inner_type: &Type) -> proc_macro2::TokenStream {
    quote! {
        if let Some((value, rest)) = <#inner_type as ::rcket::Lex>::lex_one(input) {
            return Some((Self::#variant_name(value), rest));
        }
    }
}
