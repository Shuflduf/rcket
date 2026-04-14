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
enum BinaryOperation {
    #[infix(Symbol::Plus)]
    #[prec(1)]
    AdditionOperation(Box<Expression>, Box<Expression>),
    #[infix(Symbol::Star)]
    #[prec(2)]
    MultiplicationOperation(Box<Expression>, Box<Expression>),
}

#[derive(Node, Debug, PartialEq)]
enum Expression {
    #[extract(Literal::Int)]
    Int(i32),
    // #[extract(Literal::Float)]
    // Float(f32),
    #[extract(Literal::String)]
    String(String),
    BinaryOperation(BinaryOperation),
}

#[derive(Node, Debug, PartialEq)]
struct VariableDeclaration(
    VarType,
    #[extract(Literal::Identifier)] String,
    #[token(Symbol::Equals)] (),
    Expression,
);

#[derive(Node, Debug, PartialEq)]
enum AssignmentOp {
    #[token(Symbol::Equals)]
    Equal,
    #[token(Symbol::PlusEquals)]
    PlusEqual,
}

#[derive(Node, Debug, PartialEq)]
struct VariableAssignment(
    #[extract(Literal::Identifier)] String,
    AssignmentOp,
    Expression,
);

#[derive(Node, Debug, PartialEq)]
enum Statement {
    VariableDeclaration(VariableDeclaration),
    VariableAssignment(VariableAssignment),
}

#[test]
fn parse_int() {
    let value = Expression::parse(&Token::lex("1225")).unwrap();
    assert_eq!(value, Expression::Int(1225));
    assert_eq!(value.to_string(), "Expression (Int (1225))");
}

#[test]
fn parse_add_operation() {
    assert_eq!(
        BinaryOperation::parse(&Token::lex("5+2"))
            .unwrap()
            .to_string(),
        "BinaryOperation (AdditionOperation (Expression (Int (5)) Expression (Int (2))))"
    );
}

#[test]
fn parse_larger_operations() {
    let node = BinaryOperation::parse(&Token::lex("5+2*7")).unwrap();
    assert_eq!(node.to_string(), "TODO");
}

#[test]
fn parse_mult_operation() {
    assert_eq!(
        BinaryOperation::parse(&Token::lex("7*3"))
            .unwrap()
            .to_string(),
        "BinaryOperation (MultiplicationOperation (Expression (Int (7)) Expression (Int (3))))"
    );
}

#[test]
fn parse_variable_dec() {
    assert_eq!(
        VariableDeclaration::parse(&Token::lex("int thing = 5"))
            .unwrap()
            .to_string(),
        "VariableDeclaration (Int thing Expression (Int (5)))"
    )
}

#[test]
fn parse_assignment() {
    assert_eq!(
        VariableAssignment::parse(&Token::lex("thing = 5"))
            .unwrap()
            .to_string(),
        "VariableAssignment (thing Equal Expression (Int (5)))"
    );
}

#[test]
fn parse_assignment_string() {
    assert_eq!(
        Statement::parse(&Token::lex("thing += \"concatination!\""))
            .unwrap()
            .to_string(),
        "Statement (VariableAssignment (thing PlusEqual Expression (String (concatination!))))"
    );
}

#[test]
fn parse_assignment_operation() {
    assert_eq!(
        VariableAssignment::parse(&Token::lex("OtherThing += 5 * 3"))
            .unwrap()
            .to_string(),
        "TODO"
    )
}
