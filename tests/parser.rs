use rcket::{Lex, Node};

#[path = "frg_lexer_types.rs"]
mod frg_lexer_types;
use frg_lexer_types::{Literal, Symbol, Token};

#[derive(Node, Debug, PartialEq)]
enum BinaryOperator {
    #[token(Symbol::Plus)]
    Add,
    #[token(Symbol::Minus)]
    Subtract,
    #[token(Symbol::Star)]
    Multiply,
    #[token(Symbol::FSlash)]
    Divide,
}

#[derive(Node, Debug, PartialEq)]
enum LiteralValue {
    #[extract(Literal::Int)]
    Int(i32),
    #[extract(Literal::Float)]
    Float(f32),
    #[extract(Literal::String)]
    Str(String),
}

#[derive(Node, Debug, PartialEq)]
enum Expression {
    LiteralValue(LiteralValue),
    BinaryOperation(BinaryOperation),
}

#[derive(Node, Debug, PartialEq)]
struct BinaryOperation(Box<Expression>, BinaryOperator, Box<Expression>);

#[test]
fn parse_int() {
    let tokens = Token::lex("1225");
    let value = LiteralValue::parse_all(&tokens).unwrap();
    assert_eq!(value, LiteralValue::Int(1225));
}

#[test]
fn parse_float() {
    let tokens = Token::lex("3.1415");
    let value = LiteralValue::parse_all(&tokens).unwrap();
    assert_eq!(
        value,
        LiteralValue::Float(
            #[allow(clippy::approx_constant)]
            3.1415
        )
    );
}

#[test]
fn parse_string() {
    let tokens = Token::lex(r#""froging it""#);
    let value = LiteralValue::parse_all(&tokens).unwrap();
    assert_eq!(value, LiteralValue::Str("froging it".to_string()));
}

#[test]
fn parse_operation() {
    let tokens = Token::lex("5+2");
    let (BinaryOperation(first, op, second), _) = BinaryOperation::parse(&tokens).unwrap();
    assert_eq!(*first, Expression::LiteralValue(LiteralValue::Int(5)));
    assert_eq!(op, BinaryOperator::Add);
    assert_eq!(*second, Expression::LiteralValue(LiteralValue::Int(2)));
}
