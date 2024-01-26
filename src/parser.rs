use std::io::{Error, ErrorKind, Result};

use crate::lexer::{Token, TokenType};
use crate::util::error;

#[derive(Debug, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multi,
    Div,
}

#[derive(Debug, Clone)]
pub enum Expr {
    FloatLiteral(String),
    NegFloatLiteral(String),
    Ident(String),
    Parameter(String),
    FunctionCall(String, Vec<Expr>),
    Expr(Box<Expr>, Operator, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Parsed {
    FunctionDecleration(Token, Vec<Token>, Expr),
    FromLoop(Expr, Expr, Expr, Expr, Vec<Parsed>),
    Declaration(Token, Expr),
    PrintExpr(Expr),
}

pub struct Parser {
    tokens: Vec<Token>,
    parsed: Vec<Parsed>,
    index: usize,
}

fn get_prec(op: &Operator) -> usize {
    match op {
        Operator::Plus | Operator::Minus => 1,
        Operator::Multi | Operator::Div => 2,
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            parsed: Vec::new(),
            index: 0,
        }
    }

    fn peek(&self, offset: usize) -> Option<&Token> {
        if self.index + offset < self.tokens.len() {
            Some(&self.tokens[self.index + offset])
        } else {
            None
        }
    }

    fn consume(&mut self) -> Result<&Token> {
        if self.index < self.tokens.len() {
            let cur = &self.tokens[self.index];
            self.index += 1;
            Ok(cur)
        } else {
            Err(error!(Other, "End of tokens!"))
        }
    }

    fn parse_expr(&mut self, min_prec: usize, is_function: bool) -> Result<Expr> {
        let mut left: Expr;
        if self.peek(0).is_some() {
            let token = self.peek(0).unwrap().clone();
            let token_type = &token.0;
            let loc = &token.1;
            if let TokenType::FloatLiteral(val) = token_type {
                left = Expr::FloatLiteral(val.to_string());
            } else if let TokenType::Ident(name) = token_type {
                if self
                    .peek(1)
                    .is_some_and(|Token(t, _)| t == &TokenType::LeftParen)
                {
                    self.consume()?;
                    self.consume()?;

                    let mut args: Vec<Expr> = Vec::new();

                    while self
                        .peek(0)
                        .is_some_and(|Token(t, _)| t != &TokenType::RightParen)
                    {
                        let arg = self.parse_expr(1, false)?;
                        args.push(arg);
                        if &self.peek(0).unwrap().0 == &TokenType::Comma {
                            self.consume()?;
                        }
                    }
                    left = Expr::FunctionCall(name.to_string(), args);
                } else {
                    if is_function {
                        left = Expr::Parameter(name.to_string());
                    } else {
                        left = Expr::Ident(name.to_string());
                    }
                }
            } else if let TokenType::Minus = token_type {
                let Some(Token(TokenType::FloatLiteral(val), _)) = self.peek(1) else {
                    return Err(error!(InvalidData, "Missing literal at {}", loc));
                };
                left = Expr::NegFloatLiteral(val.to_string());
                self.consume()?;
            } else if let TokenType::LeftParen = token_type {
                self.consume()?;
                left = self.parse_expr(1, is_function)?;
            } else {
                return Err(error!(InvalidData, "Unexpected token: {:?}", token));
            }
            self.consume()?;
        } else {
            return Err(error!(UnexpectedEof, "End of tokens!"));
        }

        loop {
            let cur = self.peek(0);

            if cur.is_none() {
                break;
            }

            let op = match &cur.unwrap().0 {
                &TokenType::Plus => Operator::Plus,
                &TokenType::Minus => Operator::Minus,
                &TokenType::Multi => Operator::Multi,
                &TokenType::Div => Operator::Div,
                _ => return Ok(left),
            };

            let prec = get_prec(&op);

            if prec < min_prec {
                break;
            }

            self.consume()?;
            let right = self.parse_expr(prec + 1, is_function)?;

            left = Expr::Expr(Box::new(left.clone()), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_from_block(&mut self) -> Result<Parsed> {
        self.consume()?;
        let min = self.parse_expr(1, false)?;
        self.consume()?;
        let max = self.parse_expr(1, false)?;
        self.consume()?;
        let ident = self.parse_expr(1, false)?;
        let step = if let Some(Token(TokenType::Keyword(keyword), _)) = self.peek(0) {
            match keyword.as_str() {
                "with" => {
                    self.consume()?;
                    self.consume()?;
                    let expr = self.parse_expr(1, false)?;
                    self.consume()?;
                    expr
                }
                _ => {
                    return Err(error!(
                        Other,
                        "Expected \"with\" keyword, got {:?} keyword", keyword
                    ));
                }
            }
        } else {
            Expr::FloatLiteral("1".to_string())
        };

        let mut block: Vec<Parsed> = Vec::new();

        while self
            .peek(0)
            .is_some_and(|Token(t, _)| t != &TokenType::RightBrace)
        {
            let t = &self.peek(0).unwrap().0;
            let loc = &self.peek(0).unwrap().1;
            match t {
                TokenType::Ident(_) => {
                    if self
                        .peek(1)
                        .is_some_and(|Token(t, _)| t == &TokenType::Equals)
                    {
                        let out = self.parse_declaration(self.peek(0).unwrap().clone())?;
                        block.push(out);
                    } else if self
                        .peek(1)
                        .is_some_and(|Token(t, _)| t == &TokenType::LeftParen)
                        && self.line_contains_equals()
                    {
                        let out = self.parse_function_declaration(self.peek(0).unwrap().clone())?;
                        block.push(out);
                    } else {
                        let out = self.parse_print()?;
                        block.push(out);
                    }
                }
                TokenType::Keyword(keyword) => {
                    if keyword.as_str() != "from" {
                        return Err(error!(Other, "Unexpected keyword {:?} at {}", keyword, loc));
                    }
                    let out = self.parse_from_block()?;
                    block.push(out);
                }
                TokenType::FloatLiteral(_) => {
                    let out = self.parse_print()?;
                    block.push(out);
                }
                TokenType::Comment => {
                    while self.peek(0).is_some() && &self.peek(0).unwrap().0 != &TokenType::Newline
                    {
                        self.consume()?;
                    }
                }
                TokenType::Newline => {
                    self.consume()?;
                }
                token => todo!("Handle: {:?}", token),
            }
        }
        self.consume()?;

        Ok(Parsed::FromLoop(min, max, ident, step, block))
    }

    fn parse_declaration(&mut self, ident: Token) -> Result<Parsed> {
        self.consume()?;
        self.consume()?;
        let expr = self.parse_expr(1, false)?;
        Ok(Parsed::Declaration(ident, expr))
    }

    fn parse_function_declaration(&mut self, ident: Token) -> Result<Parsed> {
        // println!("{:?}", &self.peek(0));
        let mut parameters: Vec<Token> = Vec::new();
        self.consume()?;
        self.consume()?;

        while self
            .peek(0)
            .is_some_and(|Token(t, _)| t != &TokenType::RightParen)
        {
            if let TokenType::Ident(_) = self.peek(0).unwrap().0 {
                parameters.push(self.peek(0).unwrap().clone());
            }
            self.consume()?;
        }
        self.consume()?;
        self.consume()?;
        let expr = self.parse_expr(1, true)?;
        Ok(Parsed::FunctionDecleration(ident, parameters, expr))
    }

    fn parse_print(&mut self) -> Result<Parsed> {
        let expr = self.parse_expr(1, false)?;

        Ok(Parsed::PrintExpr(expr))
    }

    fn line_contains_equals(&self) -> bool {
        for i in 0..self.tokens.len() - self.index - 1 {
            if !self.peek(i).is_some() {
                return false;
            }

            let t = &self.peek(i).unwrap().0;

            if t == &TokenType::Newline {
                return false;
            } else if t == &TokenType::Equals {
                return true;
            }
        }

        return false;
    }

    pub fn parse(&mut self) -> Result<Vec<Parsed>> {
        // println!("Tokens: {:?}", &self.tokens);
        while let Some(Token(token_type, loc)) = self.peek(0) {
            let token = self.peek(0).unwrap().clone();
            match token_type {
                TokenType::Ident(_) => {
                    if self
                        .peek(1)
                        .is_some_and(|Token(t, _)| t == &TokenType::Equals)
                    {
                        let out = self.parse_declaration(token)?;
                        self.parsed.push(out);
                    } else if self
                        .peek(1)
                        .is_some_and(|Token(t, _)| t == &TokenType::LeftParen)
                        && self.line_contains_equals()
                    {
                        let out = self.parse_function_declaration(token)?;
                        self.parsed.push(out);
                    } else {
                        let out = self.parse_print()?;
                        self.parsed.push(out);
                    }
                }
                TokenType::Keyword(keyword) => {
                    if keyword.as_str() != "from" {
                        return Err(error!(Other, "Unexpected keyword: {}", keyword));
                    }
                    let out = self.parse_from_block()?;
                    self.parsed.push(out);
                }
                TokenType::FloatLiteral(_) => {
                    let out = self.parse_print()?;
                    self.parsed.push(out);
                }
                TokenType::Comment => {
                    while let Some(Token(TokenType::Newline, _)) = self.peek(0) {
                        self.consume()?;
                    }
                }
                TokenType::Newline => {
                    self.consume()?;
                }
                TokenType::RightParen => {
                    eprintln!("{:?}", loc);
                    self.consume()?;
                }
                token => todo!("Handle: {:?}", token),
            };
        }

        Ok(self.parsed.to_vec())
    }
}
