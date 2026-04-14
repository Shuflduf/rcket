use rcket::{Lex, Node};
use std::io;

// -- Lexer tokens --
#[derive(Lex, Debug, PartialEq, Clone)]
enum Symbol {
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    FSlash,
}

#[derive(Lex, Debug, PartialEq, Clone)]
enum Literal {
    #[regex(r"\d+")]
    Int(i32),
    #[regex(r"\d+\.\d+")]
    Float(f32),
}

#[derive(Lex, Debug, PartialEq, Clone)]
enum Token {
    Symbol(Symbol),
    Literal(Literal),
}

// -- AST nodes --
#[derive(Node, Debug, PartialEq)]
enum Expression {
    #[extract(Literal::Int)]
    Int(i32),
    #[extract(Literal::Float)]
    Float(f32),
    BinaryOperation(BinaryOperation),
}

#[derive(Node, Debug, PartialEq)]
enum BinaryOperation {
    #[infix(Symbol::Plus)]
    #[prec(1)]
    Addition(Box<Expression>, Box<Expression>),

    #[infix(Symbol::Minus)]
    #[prec(1)]
    Subtraction(Box<Expression>, Box<Expression>),

    #[infix(Symbol::Star)]
    #[prec(2)]
    Multiplication(Box<Expression>, Box<Expression>),

    #[infix(Symbol::FSlash)]
    #[prec(2)]
    Division(Box<Expression>, Box<Expression>),
}

fn main() {
    println!("Welcome to the rcket demo!");
    println!("Enter an expression using binary operations (+, -, *, /)");
    println!("Example: `5 + (2 - 4) * 7`");
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let tokens = Token::lex(&input);
        let ast = Expression::parse(&tokens).unwrap();
        println!("Tokens: {tokens:?}");
        println!("AST: {ast}");
    }
}
