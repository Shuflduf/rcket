use rcket::{Node, lexer::lex};

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
enum Literal {
    #[extract(Literal::Int)]
    Int(i32),
    #[extract(Literal::Float)]
    Float(f32),
    #[extract(Literal::String)]
    Str(String),
}

#[derive(Node, Debug, PartialEq)]
enum Expression {
    Literal(Literal),
    BinaryOperation(BinaryOperation),
}

#[derive(Node, Debug, PartialEq)]
struct BinaryOperation(Box<Expression>, BinaryOperator, Box<Expression>);

#[test]
fn parse_int() {
    let tokens = lex("1225");
    let value = Literal::parse_all(&tokens).unwrap();
    assert_eq!(value, Literal::Int(1225));
}

#[test]
fn parse_float() {
    let tokens = lex("3.1415");
    let value = Literal::parse_all(&tokens).unwrap();
    assert_eq!(
        value,
        Literal::Float(
            #[allow(clippy::approx_constant)]
            3.1415
        )
    );
}

#[test]
fn parse_string() {
    let tokens = lex(r#""froging it""#);
    let value = Literal::parse_all(&tokens).unwrap();
    assert_eq!(value, Literal::Str("froging it".to_string()));
}

#[test]
fn parse_operation() {
    let tokens = lex("5+2");
    let (BinaryOperation(first, op, second), _) = BinaryOperation::parse(&tokens).unwrap();
    assert_eq!(*first, Expression::Literal(Literal::Int(5)));
    assert_eq!(op, BinaryOperator::Add);
    assert_eq!(*second, Expression::Literal(Literal::Int(2)));
}
