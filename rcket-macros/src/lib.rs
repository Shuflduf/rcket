extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod lex;
mod node;

#[proc_macro_error]
#[proc_macro_derive(Node, attributes(token, extract, node, prec))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    node::derive_node(input)
}

#[proc_macro_derive(Lex, attributes(token, regex, seq, choice))]
#[proc_macro_error]
pub fn derive_lex(input: TokenStream) -> TokenStream {
    lex::derive_lex(input)
}
