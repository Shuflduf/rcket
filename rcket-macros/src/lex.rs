use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DeriveInput, Ident, LitStr, Type,
    parse_macro_input, punctuated::Punctuated,
};

use crate::single_unnamed_field;

pub(crate) struct LexPattern {
    kind: LexPatternKind,
    lit: LitStr,
}

pub(crate) enum LexPatternKind {
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

pub(crate) fn derive_lex(input: TokenStream) -> TokenStream {
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
