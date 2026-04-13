![Hackatime Hours](https://hackatime-badge.hackclub.com/U07950S3GMC/rcket)

# 🚀
rcket is a general-purpose, minimal-dependency parsing library made in Rust, for Rust.

It's as simple as defining a list of tokens, and defining an Abstract Syntax Tree using structs and enums.

> [!NOTE]
> This library is a work in progress.
>
> Expect some stuff to not work.

# Demo
While this is a library, you can find a demo that uses this library in the [Github Releases](https://github.com/Shuflduf/rcket/releases/latest).

It is a simple expression parser and interpreter, the code for which is shown in these examples and in [/interpreter-example]

# Installation
You can add this library to your project through the terminal:
```sh
# In a Rust project
cargo add rcket
```

Or through `Cargo.toml`:

```toml
[dependencies]
rcket = "1.0.0" # or whatever the latest version is
```

# Lexer
A lexer turns raw text into a series of tokens.

## Definition
```rs
use rcket::{Lex, Node};

#[derive(Lex, Debug, PartialEq, Clone)]
pub enum Symbol {
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
pub enum Literal {
    #[regex(r"\d+")]
    Int(i32),
    #[regex(r"\d+\.\d+")]
    Float(f32),
}

#[derive(Lex, Debug, PartialEq, Clone)]
pub enum Token {
    Symbol(Symbol),
    Literal(Literal),
}
```

## Available Attributes
- `#[token()]`: Matches text directly
  - Doesn't return the value inside, just checks for matches
- `#[regex()]`: Matches text using regex
  - Returns the parsed matched values
- `#[choice()]`: Matches any of the attributes inside
  - All choices have to either return a value of the same type, or not return anything at all
  - `#[choice( token("map"), token("obj"), token("hashmap"), token("dict") )] Object`
- `#[seq()]`: Matches the attributes inside in order
  - Returns a `Vec` of returned values, unless there's only one element, then it is returned directly
  - `#[seq(token("\""), regex(r#"[^"]*"#), token("\""))] String(String)`

## Usage
```rs
let input = "12 + 25";
let tokens = Token::lex(input);
assert_eq!(tokens, vec![
  Token::Literal(Literal::Int(12)),
  Token::Symbol(Symbol::Plus),
  Token::Literal(Literal::Int(25)),
]);
```

# Parser
A parser turns a list of tokens into a tree-like structure representing the code

## Definition

```rs
use rcket::{Lex, Node};

#[derive(Node, Debug, PartialEq)]
enum Expression {
    #[extract(Literal::Int)]
    Int(i32),
    #[extract(Literal::Float)]
    Float(f32),
    BinaryOperation(BinaryOperation),
}

#[derive(Node, Debug, PartialEq)]
// Box<Expression> is used to avoid Rust recursion errors
struct AdditionOperation(Box<Expression>, #[token(Symbol::Plus)] (), Box<Expression>);

#[derive(Node, Debug, PartialEq)]
struct MultiplicationOperation(Box<Expression>, #[token(Symbol::Star)] (), Box<Expression>);

// TODO: sub and divide operations

#[derive(Node, Debug, PartialEq)]
enum BinaryOperation {
    #[prec(1)]
    AdditionOperation(AdditionOperation),
    #[prec(2)]
    MultiplicationOperation(MultiplicationOperation),
}
```

## Available Attributes
- `#[extract()]`: Inserts the inner value of a token into the AST node
  - The types of the token and node have to be the same
- `#[token()]`: Checks for a token's existence
- `#[prec()]`: Precedent to allow some operations to take priority over others
  - Currently WIP

## Usage
```rs
let ast = Expression::parse(&Token::lex("12 + 25")).unwrap();
assert_eq!(value, Expression::BinaryOperation(BinaryOperation::AdditionOperation(Box::new(Expression::Int(12)), (), Box::new(Expression::Int(25)))));
assert_eq!(value.to_string(), "Expression (BinaryOperation (AdditionOperation (Expression (Int (12)) Expression (Int (25)))))");
```

