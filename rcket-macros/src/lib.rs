extern crate proc_macro;
use proc_macro::TokenStream;

mod lex;
mod node;

#[proc_macro_derive(Node, attributes(token, extract))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    node::derive_node(input)
}

#[proc_macro_derive(Lex, attributes(token, regex, seq, choice))]
pub fn derive_lex(input: TokenStream) -> TokenStream {
    lex::derive_lex(input)
}
