use crate::token::{Token, TokenType};

pub struct Lexer {
    source: Vec<char>,
    current: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        while !self.is_at_end() {
            if let Some(token) = self.scan_token() {
                tokens.push(token)
            }
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            span: (self.current, self.current),
        });
        tokens
    }
}

impl Lexer {
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.source.get(self.current).copied().unwrap_or('\0');
        self.current += 1;
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\t' | '\n' | '\r' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn scan_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        if self.is_at_end() {
            return None;
        }

        let start = self.current;
        let ch = self.advance();

        let token_type = match ch {
            '+' => TokenType::Plus,
            '*' => TokenType::Star,
            '=' => TokenType::Equals,
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '0'..='9' => {
                self.current -= 1;
                TokenType::Number(self.read_number())
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                self.current -= 1;
                let ident = self.read_identifier();

                // Check if keyword
                match ident.as_str() {
                    "let" => TokenType::Let,
                    "return" => TokenType::Return,
                    "public" => TokenType::Public,
                    "private" => TokenType::Private,
                    "const" => TokenType::Const,
                    _ => TokenType::Identifier(ident),
                }
            }
            _ => panic!("Unexpected character: {}", ch),
        };

        Some(Token {
            token_type,
            span: (start, self.current),
        })
    }

    fn read_number(&mut self) -> i32 {
        let start = self.current;

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        let num_str: String = self.source[start..self.current].iter().collect();
        num_str.parse().unwrap()
    }

    fn read_identifier(&mut self) -> String {
        let start = self.current;

        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        self.source[start..self.current].iter().collect()
    }
}
