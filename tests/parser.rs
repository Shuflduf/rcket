use rcket::{Lex, Node};

#[path = "frg_lexer_types.rs"]
mod frg_lexer_types;
use frg_lexer_types::{Keyword, Literal, Symbol, Token};

#[derive(Node, Debug, PartialEq)]
enum VarType {
    #[token(Keyword::Int)]
    Int,
    #[token(Keyword::Float)]
    Float,
    #[token(Keyword::Str)]
    Str,
    #[token(Keyword::Bool)]
    Bool,
}

#[derive(Node, Debug, PartialEq)]
enum Identifier {
    #[extract(Literal::Identifier)]
    Name(String),
}
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
struct BinaryOperation(Box<Expression>, BinaryOperator, Box<Expression>);

#[derive(Node, Debug, PartialEq)]
enum Expression {
    #[extract(Literal::Int)]
    Int(i32),
    #[extract(Literal::Float)]
    Float(f32),
    #[extract(Literal::String)]
    String(String),
    BinaryOperation(BinaryOperation),
}

#[derive(Node, Debug, PartialEq)]
struct VariableDeclaration(VarType, Identifier, #[token(Symbol::Equals)] (), Expression);

#[derive(Node, Debug, PartialEq)]
enum Statement {
    VariableDeclaration(VariableDeclaration),
}

#[test]
fn parse_int() {
    let value = Expression::parse(&Token::lex("1225")).unwrap();
    assert_eq!(value, Expression::Int(1225));
}

#[test]
fn parse_float() {
    let value = Expression::parse(&Token::lex("3.1415")).unwrap();
    assert_eq!(
        value,
        Expression::Float(
            #[allow(clippy::approx_constant)]
            3.1415
        )
    );
}

#[test]
fn parse_string() {
    let value = Expression::parse(&Token::lex(r#""froging it""#)).unwrap();
    assert_eq!(value, Expression::String("froging it".to_string()));
}

#[test]
fn parse_operation() {
    let (BinaryOperation(first, op, second), _) =
        BinaryOperation::parse_one(&Token::lex("5+2")).unwrap();
    assert_eq!(*first, Expression::Int(5));
    assert_eq!(op, BinaryOperator::Add);
    assert_eq!(*second, Expression::Int(2));
}

#[test]
fn display_operation() {
    let (node, _) = BinaryOperation::parse_one(&Token::lex("5+2")).unwrap();
    panic!("{node}");
    assert_eq!(node.to_string(), "(BinaryOperation (Int 5) Add (Int 2))");
}
