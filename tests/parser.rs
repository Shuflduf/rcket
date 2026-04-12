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
struct AdditionOperation(Box<Expression>, #[token(Symbol::Plus)] (), Box<Expression>);

#[derive(Node, Debug, PartialEq)]
struct MultiplicationOperation(Box<Expression>, #[token(Symbol::Star)] (), Box<Expression>);

#[derive(Node, Debug, PartialEq)]
enum BinaryOperation {
    // #[prec(1)]
    AdditionOperation(AdditionOperation),
    // #[prec(2)]
    MultiplicationOperation(MultiplicationOperation),
}

#[derive(Node, Debug, PartialEq)]
enum Expression {
    #[extract(Literal::Int)]
    Int(i32),
    // #[extract(Literal::Float)]
    // Float(f32),
    // #[extract(Literal::String)]
    // String(String),
    // BinaryOperation(BinaryOperation),
}

#[derive(Node, Debug, PartialEq)]
struct VariableDeclaration(
    VarType,
    #[extract(Literal::Identifier)] String,
    #[token(Symbol::Equals)] (),
    Expression,
);

// #[derive(Node, Debug, PartialEq)]
// enum Statement {
//     VariableDeclaration(VariableDeclaration),
// }

#[test]
fn parse_int() {
    let value = Expression::parse(&Token::lex("1225")).unwrap();
    // println!("{:#?}", Expression::Int(1225));
    assert_eq!(value, Expression::Int(1225));
    assert_eq!(value.to_string(), "(Expression (Int (1225)))");
}

// #[test]
// fn parse_float() {
//     let value = Expression::parse(&Token::lex("3.1415")).unwrap();
//     assert_eq!(
//         value,
//         Expression::Float(
//             #[allow(clippy::approx_constant)]
//             3.1415
//         )
//     );
// }

// #[test]
// fn parse_string() {
//     let value = Expression::parse(&Token::lex(r#""froging it""#)).unwrap();
//     assert_eq!(value.to_string(), "(Expression (String (froging it)))");
// }

#[test]
fn parse_add_operation() {
    let node = BinaryOperation::parse(&Token::lex("5+2")).unwrap();
    assert_eq!(
        node.to_string(),
        "(BinaryOperation (AdditionOperation (Expression (Int (5))) (Expression (Int (2)))))"
    );
}

#[test]
fn parse_mult_operation() {
    let node = BinaryOperation::parse(&Token::lex("7*3")).unwrap();
    println!("{node:#?}");
    assert_eq!(
        node.to_string(),
        "(BinaryOperation (MultiplicationOperation (Expression (Int (7))) (Expression (Int (3)))))"
    );
}

#[test]
fn parse_variable_dec() {
    let node = VariableDeclaration::parse(&Token::lex("int thing = 5")).unwrap();
    assert_eq!(
        node.to_string(),
        "(VariableDeclaration (Int thing (Expression (Int (5))))"
    )
}
