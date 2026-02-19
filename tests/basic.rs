use rcket::Node;

#[derive(Node, Debug, PartialEq)]
enum BinaryOperator {
    #[token("+")]
    Add,
    #[token("-")]
    Subtract,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
}

#[derive(Node, Debug, PartialEq)]
enum Literal {
    #[regex(r"\d+")]
    Int(i32),
    #[regex(r"\d+\.\d+")]
    Float(f32),
    #[seq(token("\""), regex(r#"[^"]*"#), token("\""))]
    String(String),
}

#[derive(Node, Debug, PartialEq)]
enum Expression {
    Literal(Literal),
}

#[derive(Node, Debug)]
struct BinaryOperation(Box<Expression>, BinaryOperator, Box<Expression>);

#[test]
fn parse_int() {
    let value = Literal::parse("1225").unwrap();
    assert_eq!(value, Literal::Int(1225));
}

#[test]
fn parse_float() {
    let value = Literal::parse("3.1415").unwrap();
    assert_eq!(value, Literal::Float(3.1415));
}

#[test]
fn parse_string() {
    let value = Literal::parse("\"froging it\"").unwrap();
    assert_eq!(value, Literal::String("froging it".to_string()));
}

#[test]
fn parse_operation() {
    let (first, op, second) = BinaryOperation::parse("5+2").unwrap();
    assert_eq!(*first, Expression::Literal(Literal::Int(5)));
    assert_eq!(op, BinaryOperator::Add);
    assert_eq!(*second, Expression::Literal(Literal::Int(2)));
}
