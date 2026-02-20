mod lexer;
mod lexer_types;

pub use rcket_macros::Node;

pub trait Node {
    type Output;
    fn parse(input: &str) -> Option<Self::Output>;
}
