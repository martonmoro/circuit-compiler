#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Let,
    Return,

    Star,
    Plus,
    Equals,

    Identifier(String),
    Number(i32),

    LeftParen,
    RightParen,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: (usize, usize), // TODO: for error messages
}
