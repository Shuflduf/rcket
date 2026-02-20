use rcket::Lex;

#[path = "frg_lexer_types.rs"]
mod frg_lexer_types;
use frg_lexer_types::{Keyword, Literal, Symbol, Token};

#[test]
fn lex_float() {
    let input = "5.8";
    let target = vec![Token::Literal(Literal::Float(5.8))];
    let output = Token::lex(input);
    assert_eq!(target, output)
}

#[test]
fn lex_declaration() {
    let input = "int value = func()";
    let target = vec![
        Token::Keyword(Keyword::Int),
        Token::Literal(Literal::Identifier("value".into())),
        Token::Symbol(Symbol::Equals),
        Token::Literal(Literal::Identifier("func".into())),
        Token::Symbol(Symbol::LeftParen),
        Token::Symbol(Symbol::RightParen),
    ];
    let output = Token::lex(input);
    assert_eq!(target, output)
}

#[test]
fn lex_int() {
    let (token, rest) = Token::lex_one("42 rest").unwrap();
    assert_eq!(token, Token::Literal(Literal::Int(42)));
    assert_eq!(rest, " rest");
}

#[test]
fn lex_keyword() {
    let (token, rest) = Token::lex_one("int foo").unwrap();
    assert_eq!(token, Token::Keyword(Keyword::Int));
    assert_eq!(rest, " foo");
}

#[test]
fn lex_string() {
    let (token, rest) = Token::lex_one(r#""hello" rest"#).unwrap();
    assert_eq!(token, Token::Literal(Literal::String("hello".into())));
    assert_eq!(rest, " rest");
}

#[test]
fn lex_full() {
    let tokens = Token::lex("int x = 42");
    assert_eq!(
        tokens,
        vec![
            Token::Keyword(Keyword::Int),
            Token::Literal(Literal::Identifier("x".into())),
            Token::Symbol(Symbol::Equals),
            Token::Literal(Literal::Int(42)),
        ]
    );
}
