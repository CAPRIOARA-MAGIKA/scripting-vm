use crate::token::{Token, TokenType};

pub struct Lexer {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, String> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_one()?;
        }
        self.tokens
            .push(Token::new(TokenType::EOF, String::new(), self.line));
        Ok(self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn matches(&mut self, c: char) -> bool {
        if self.is_at_end() || self.source[self.current] != c {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn add_token(&mut self, kind: TokenType) {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(kind, lexeme, self.line));
    }

    fn scan_one(&mut self) -> Result<(), String> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let k = if self.matches('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(k);
            }
            '=' => {
                let k = if self.matches('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(k);
            }
            '<' => {
                let k = if self.matches('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(k);
            }
            '>' => {
                let k = if self.matches('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(k);
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string()?,
            d if d.is_ascii_digit() => self.number()?,
            a if a.is_ascii_alphabetic() || a == '_' => self.identifier(),
            _ => {
                return Err(format!(
                    "Unexpected character '{}' at line {}",
                    c, self.line
                ))
            }
        }
        Ok(())
    }

    fn string(&mut self) -> Result<(), String> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(format!("Unterminated string at line {}", self.line));
        }
        self.advance(); // closing "
        let s: String = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect();
        let lex: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::string(lex, s, self.line));
        Ok(())
    }

    fn number(&mut self) -> Result<(), String> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let s: String = self.source[self.start..self.current].iter().collect();
        let n: f64 = s
            .parse()
            .map_err(|_| format!("Invalid number {} at line {}", s, self.line))?;
        let lex: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::number(lex, n, self.line));
        Ok(())
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let s: String = self.source[self.start..self.current].iter().collect();
        let kind = TokenType::keyword(&s).unwrap_or(TokenType::Identifier);
        self.add_token(kind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(src: &str) -> Vec<Token> {
        Lexer::new(src).scan_tokens().expect("lex")
    }

    #[test]
    fn lexes_punctuation() {
        let toks = lex("(){},.;-+*");
        let kinds: Vec<TokenType> = toks.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBrace,
                TokenType::RightBrace,
                TokenType::Comma,
                TokenType::Dot,
                TokenType::Semicolon,
                TokenType::Minus,
                TokenType::Plus,
                TokenType::Star,
                TokenType::EOF,
            ]
        );
    }

    #[test]
    fn lexes_two_char_operators() {
        let toks = lex("!= == <= >= ! = < >");
        let kinds: Vec<TokenType> = toks.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenType::BangEqual,
                TokenType::EqualEqual,
                TokenType::LessEqual,
                TokenType::GreaterEqual,
                TokenType::Bang,
                TokenType::Equal,
                TokenType::Less,
                TokenType::Greater,
                TokenType::EOF,
            ]
        );
    }

    #[test]
    fn lexes_numbers() {
        let toks = lex("42 3.14");
        assert_eq!(toks[0].kind, TokenType::Number);
        assert_eq!(toks[1].kind, TokenType::Number);
    }

    #[test]
    fn lexes_string() {
        let toks = lex("\"hi\"");
        assert_eq!(toks[0].kind, TokenType::String);
        assert_eq!(toks[0].lexeme, "\"hi\"");
    }

    #[test]
    fn lexes_keywords_and_identifiers() {
        let toks = lex("var x = if true");
        let kinds: Vec<TokenType> = toks.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenType::Var,
                TokenType::Identifier,
                TokenType::Equal,
                TokenType::If,
                TokenType::True,
                TokenType::EOF,
            ]
        );
    }

    #[test]
    fn skips_line_comments() {
        let toks = lex("// hi\n42");
        assert_eq!(toks[0].kind, TokenType::Number);
        assert_eq!(toks[0].line, 2);
    }

    #[test]
    fn tracks_lines() {
        let toks = lex("a\nb\nc");
        assert_eq!(toks[0].line, 1);
        assert_eq!(toks[1].line, 2);
        assert_eq!(toks[2].line, 3);
    }

    #[test]
    fn unterminated_string_is_error() {
        assert!(Lexer::new("\"abc").scan_tokens().is_err());
    }
}
