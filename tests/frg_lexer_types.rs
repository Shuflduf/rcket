use rcket::Lex;

#[derive(Lex, Debug, PartialEq, Clone)]
pub enum Keyword {
    #[token("struct")]
    Struct,
    #[token("void")]
    Void,
    #[token("int")]
    Int,
    #[token("float")]
    Float,
    #[token("bool")]
    Bool,
    #[token("str")]
    Str,
    #[choice(token("vec"), token("arr"), token("array"), token("list"))]
    Vec,
    #[choice(
        token("map"),
        token("obj"),
        token("hashmap"),
        token("dict"),
        token("dictionary")
    )]
    Map,
    #[token("set")]
    Set,
    #[regex(r"r?e?t?u?r?n?")]
    Return,
    #[token("if")]
    If,
    #[token("elif")]
    Elif,
    #[token("else")]
    Else,
}

#[derive(Lex, Debug, PartialEq, Clone)]
pub enum Symbol {
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("!=")]
    NotEquals,
    #[token("==")]
    DoubleEquals,
    #[token("=")]
    Equals,
    #[token(",")]
    Comma,
    #[token("&")]
    Ampersand,
    #[token("+=")]
    PlusEquals,
    #[token("+")]
    Plus,
    #[token("-=")]
    MinusEquals,
    #[token("-")]
    Minus,
    #[token("/=")]
    FSlashEquals,
    #[token("/")]
    FSlash,
    #[token("*=")]
    StarEquals,
    #[token("*")]
    Star,
    #[token(":")]
    Colon,
    #[token(".")]
    Period,
    #[token("<=")]
    LessThanOrEqual,
    #[token("<")]
    LessThan,
    #[token(">=")]
    GreaterThanOrEqual,
    #[token(">")]
    GreaterThan,
    #[token("!")]
    Exclamation,
}

#[derive(Lex, Debug, PartialEq, Clone)]
pub enum Literal {
    #[regex(r"\d+\.\d+")]
    Float(f32),
    #[regex(r"\d+")]
    Int(i32),
    #[seq(token("\""), regex(r#"[^"]*"#), token("\""))]
    String(String),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier(String),
}

#[derive(Lex, Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    Literal(Literal),
}
