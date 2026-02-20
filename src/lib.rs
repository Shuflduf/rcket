pub use rcket_macros::{Lex, Node};

pub trait Node {
    type Token;
    type Output;
    fn parse_one(tokens: &[Self::Token]) -> Option<(Self::Output, &[Self::Token])>;

    fn parse(tokens: &[Self::Token]) -> Option<Self::Output> {
        let (result, rest) = Self::parse_one(tokens)?;
        if rest.is_empty() { Some(result) } else { None }
    }
}

pub trait Lex: Sized {
    fn lex_one(input: &str) -> Option<(Self, &str)>;

    fn lex(input: &str) -> Vec<Self> {
        let mut tokens = Vec::new();
        let mut remaining = input;
        loop {
            remaining = remaining.trim_start();
            if remaining.is_empty() {
                break;
            }
            match Self::lex_one(remaining) {
                Some((token, rest)) => {
                    tokens.push(token);
                    remaining = rest;
                }
                None => {
                    let skip_to = remaining
                        .char_indices()
                        .nth(1)
                        .map(|(index, _)| index)
                        .unwrap_or(remaining.len());
                    remaining = &remaining[skip_to..];
                }
            }
        }
        tokens
    }
}
