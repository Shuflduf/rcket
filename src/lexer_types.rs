#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Struct,
    Void,
    Int,
    Float,
    Bool,
    Str,
    Vec,
    Map,
    Set,
    Return,
    If,
    Elif,
    Else,
    Loop,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Symbol {
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Equals,
    NotEquals,
    DoubleEquals,
    Comma,
    Ampersand,
    Plus,
    PlusEquals,
    Minus,
    MinusEquals,
    FSlash,
    FSlashEquals,
    Star,
    StarEquals,
    Colon,
    Period,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Exclamation,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Int(i32),
    Float(f32),
    String(String),
    Identifier(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    Literal(Literal),
}
