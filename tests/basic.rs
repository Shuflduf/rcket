use rcket::Node;

#[derive(Node, Debug)]
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

#[derive(Node, Debug)]
enum Literal {
    #[regex(r"\d+")]
    Int(i32),
    #[regex(r"\d+\.\d+")]
    Float(f32),
}

#[derive(Node, Debug)]
enum Expression {
    Literal(Literal),
}

#[derive(Node, Debug)]
struct BinaryOperation(Box<Expression>, BinaryOperator, Box<Expression>);

#[test]
fn parse_token() {
    let (first, op, second) = BinaryOperation::parse("5+2").unwrap();
    assert_eq!(first, Literal::Int(5));
    assert_eq!(op, BinaryOperator::Add);
    assert_eq!(second, Literal::Int(2));
}
