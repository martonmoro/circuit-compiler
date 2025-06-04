/*
program = statement*
statement = "public" IDENT
          | "private" IDENT
          | "const" IDENT "=" NUMBER
          | "let" IDENT "=" expr
          | "return" expr
expr = term ("+" term | "*" term)*
term = IDENT | NUMBER | "(" expr ")"
*/

use crate::ast::{Expr, Program, Stmt};
use crate::token::{Token, TokenType};

use std::mem::discriminant;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    // program = statement*
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }
        Ok(Program { statements })
    }
}

impl Parser {
    // statement = "public" IDENT
    //       | "private" IDENT
    //       | "const" IDENT "=" NUMBER
    //       | "let" IDENT "=" expr
    //       | "return" expr
    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            TokenType::Public => self.parse_public_stmt(),
            TokenType::Private => self.parse_private_stmt(),
            TokenType::Const => self.parse_const_stmt(),
            TokenType::Let => self.parse_let_stmt(),
            TokenType::Return => self.parse_return_stmt(),
            _ => Err(ParseError {
                message: format!("Expected statement, found {:?}", self.peek()),
            }),
        }
    }

    // "public" IDENT
    fn parse_public_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::Public)?;
        let name = self.expect_identifier()?;
        Ok(Stmt::PublicInput { name })
    }

    // "private" IDENT
    fn parse_private_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::Private)?;
        let name = self.expect_identifier()?;
        Ok(Stmt::PrivateInput { name })
    }

    // "const" IDENT "=" NUMBER
    fn parse_const_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::Const)?;
        let name = self.expect_identifier()?;
        self.consume(TokenType::Equals)?;
        let value = self.expect_number()?;
        Ok(Stmt::ConstDecl { name, value })
    }

    // "let" IDENT "=" expr
    fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::Let)?;
        let name = self.expect_identifier()?;
        self.consume(TokenType::Equals)?;
        let expr = self.parse_expr()?;
        Ok(Stmt::Let { name, expr })
    }

    // "return" expr
    fn parse_return_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::Return)?;
        let expr = self.parse_expr()?;
        Ok(Stmt::Return(expr))
    }

    // expr = term ("+" term | "*" term)*
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;

        while matches!(self.peek(), TokenType::Plus | TokenType::Star) {
            let op = self.advance()?;
            let right = self.parse_term()?;

            left = match op.token_type {
                TokenType::Plus => Expr::Add(Box::new(left), Box::new(right)),
                TokenType::Star => Expr::Mul(Box::new(left), Box::new(right)),
                _ => unreachable!(),
            };
        }

        Ok(left)
    }

    // term = IDENT | NUMBER | "(" expr ")"
    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance()?;

        match token.token_type {
            TokenType::Identifier(name) => Ok(Expr::Var(name)),
            TokenType::Number(n) => Ok(Expr::Literal(n)),
            TokenType::LeftParen => {
                let expr = self.parse_expr()?;
                self.consume(TokenType::RightParen)?;
                Ok(expr)
            }
            _ => Err(ParseError {
                message: format!(
                    "Expected identifier, number, or '(', found {:?}",
                    token.token_type
                ),
            }),
        }
    }
}

impl Parser {
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || *self.peek() == TokenType::Eof
    }

    fn peek(&self) -> &TokenType {
        self.tokens
            .get(self.current)
            .map(|token| &token.token_type)
            .unwrap_or(&TokenType::Eof)
    }

    fn advance(&mut self) -> Result<Token, ParseError> {
        if let Some(token) = self.tokens.get(self.current) {
            self.current += 1;
            Ok(token.clone())
        } else {
            Err(ParseError {
                message: "Unexpected end of input".to_string(),
            })
        }
    }

    fn consume(&mut self, expected: TokenType) -> Result<Token, ParseError> {
        if discriminant(self.peek()) == discriminant(&expected) {
            self.advance()
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, self.peek()),
            })
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match self.advance()?.token_type {
            TokenType::Identifier(name) => Ok(name),
            other => Err(ParseError {
                message: format!("Expected identifier, found {:?}", other),
            }),
        }
    }

    fn expect_number(&mut self) -> Result<i32, ParseError> {
        match self.advance()?.token_type {
            TokenType::Number(n) => Ok(n),
            other => Err(ParseError {
                message: format!("Expected number, found {:?}", other),
            }),
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}
