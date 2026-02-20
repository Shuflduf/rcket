use crate::lexer_types::*;

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut chars = input.char_indices().peekable();

    while let Some(&(_, current_char)) = chars.peek() {
        if current_char.is_whitespace() {
            chars.next();
            continue;
        }

        if current_char == '"' {
            chars.next();
            let mut string_content = String::new();
            while let Some((_, character)) = chars.next() {
                if character == '"' {
                    break;
                }
                string_content.push(character);
            }
            tokens.push(Token::Literal(Literal::String(string_content)));
            continue;
        }

        if current_char.is_ascii_digit() {
            let mut number_str = String::new();
            let mut is_float = false;
            while let Some(&(_, character)) = chars.peek() {
                if character.is_ascii_digit() {
                    number_str.push(character);
                    chars.next();
                } else if character == '.' && !is_float {
                    is_float = true;
                    number_str.push(character);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(if is_float {
                Token::Literal(Literal::Float(number_str.parse().unwrap()))
            } else {
                Token::Literal(Literal::Int(number_str.parse().unwrap()))
            });
            continue;
        }

        if current_char.is_alphabetic() || current_char == '_' {
            let mut identifier = String::new();
            while let Some(&(_, character)) = chars.peek() {
                if character.is_alphanumeric() || character == '_' {
                    identifier.push(character);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(match identifier.as_str() {
                "struct" => Token::Keyword(Keyword::Struct),
                "void" => Token::Keyword(Keyword::Void),
                "int" => Token::Keyword(Keyword::Int),
                "float" => Token::Keyword(Keyword::Float),
                "bool" => Token::Keyword(Keyword::Bool),
                "str" => Token::Keyword(Keyword::Str),
                "vec" => Token::Keyword(Keyword::Vec),
                "map" => Token::Keyword(Keyword::Map),
                "set" => Token::Keyword(Keyword::Set),
                "return" => Token::Keyword(Keyword::Return),
                "if" => Token::Keyword(Keyword::If),
                "elif" => Token::Keyword(Keyword::Elif),
                "else" => Token::Keyword(Keyword::Else),
                "loop" => Token::Keyword(Keyword::Loop),
                _ => Token::Literal(Literal::Identifier(identifier)),
            });
            continue;
        }

        chars.next();
        let next_char = chars.peek().map(|&(_, character)| character);
        let symbol = match (current_char, next_char) {
            ('!', Some('=')) => {
                chars.next();
                Symbol::NotEquals
            }
            ('=', Some('=')) => {
                chars.next();
                Symbol::DoubleEquals
            }
            ('+', Some('=')) => {
                chars.next();
                Symbol::PlusEquals
            }
            ('-', Some('=')) => {
                chars.next();
                Symbol::MinusEquals
            }
            ('/', Some('=')) => {
                chars.next();
                Symbol::FSlashEquals
            }
            ('*', Some('=')) => {
                chars.next();
                Symbol::StarEquals
            }
            ('<', Some('=')) => {
                chars.next();
                Symbol::LessThanOrEqual
            }
            ('>', Some('=')) => {
                chars.next();
                Symbol::GreaterThanOrEqual
            }
            ('{', _) => Symbol::LeftBrace,
            ('}', _) => Symbol::RightBrace,
            ('(', _) => Symbol::LeftParen,
            (')', _) => Symbol::RightParen,
            ('[', _) => Symbol::LeftBracket,
            (']', _) => Symbol::RightBracket,
            ('=', _) => Symbol::Equals,
            ('!', _) => Symbol::Exclamation,
            (',', _) => Symbol::Comma,
            ('&', _) => Symbol::Ampersand,
            ('+', _) => Symbol::Plus,
            ('-', _) => Symbol::Minus,
            ('/', _) => Symbol::FSlash,
            ('*', _) => Symbol::Star,
            (':', _) => Symbol::Colon,
            ('.', _) => Symbol::Period,
            ('<', _) => Symbol::LessThan,
            ('>', _) => Symbol::GreaterThan,
            _ => continue,
        };
        tokens.push(Token::Symbol(symbol));
    }

    tokens
}
