pub mod lexer;
pub mod lexer_types;

pub use rcket_macros::Node;

pub trait Node {
    type Output;
    fn parse(tokens: &[lexer_types::Token]) -> Option<(Self::Output, &[lexer_types::Token])>;

    fn parse_all(tokens: &[lexer_types::Token]) -> Option<Self::Output> {
        let (result, rest) = Self::parse(tokens)?;
        if rest.is_empty() { Some(result) } else { None }
    }
}
