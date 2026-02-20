use rcket::{lexer::lex, lexer_types::*};

#[test]
fn lex_float() {
    let input = "5.8";
    let target = vec![Token::Literal(Literal::Float(5.8))];
    let output = lex(input);
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
    let output = lex(input);
    assert_eq!(target, output)
}
