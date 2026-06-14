use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One- or two-character tokens
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals
    Identifier, String, Number,

    // Keywords
    And, Break, Class, Continue, Else, False, Fn, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    EOF,
}

impl TokenType {
    pub fn keyword(lexeme: &str) -> Option<TokenType> {
        match lexeme {
            "and" => Some(TokenType::And),
            "break" => Some(TokenType::Break),
            "class" => Some(TokenType::Class),
            "continue" => Some(TokenType::Continue),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "fn" => Some(TokenType::Fn),
            "for" => Some(TokenType::For),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
    None,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: String, line: usize) -> Self {
        Self { kind, lexeme, literal: Literal::None, line }
    }

    pub fn number(lexeme: String, n: f64, line: usize) -> Self {
        Self { kind: TokenType::Number, lexeme, literal: Literal::Number(n), line }
    }

    pub fn string(lexeme: String, s: String, line: usize) -> Self {
        Self { kind: TokenType::String, lexeme, literal: Literal::Str(s), line }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}({})", self.kind, self.lexeme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_display_includes_lexeme() {
        let t = Token::new(TokenType::Number, "42".into(), 1);
        assert_eq!(t.lexeme, "42");
        assert_eq!(t.line, 1);
    }

    #[test]
    fn token_type_equality() {
        assert_eq!(TokenType::Plus, TokenType::Plus);
        assert_ne!(TokenType::Plus, TokenType::Minus);
    }
}
