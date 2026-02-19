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
}

#[derive(Node, Debug, PartialEq)]
enum Expression {
    Literal(Literal),
}

#[derive(Node, Debug)]
struct BinaryOperation(Box<Expression>, BinaryOperator, Box<Expression>);

#[test]
fn parse_int() {
    let value = Literal::parse("5").unwrap();
    assert_eq!(value, Literal::Int(5));
}

#[test]
fn parse_operation() {
    let (first, op, second) = BinaryOperation::parse("5+2").unwrap();
    assert_eq!(*first, Expression::Literal(Literal::Int(5)));
    assert_eq!(op, BinaryOperator::Add);
    assert_eq!(*second, Expression::Literal(Literal::Int(2)));
}
