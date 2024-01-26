use std::io::{Error, ErrorKind, Result};

use crate::lexer::Token;
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
    FunctionDecleration(String, Vec<String>, Expr),
    FromLoop(Expr, Expr, Expr, Vec<Parsed>),
    Declaration(String, Expr),
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
            if let Token::FloatLiteral(val) = token {
                left = Expr::FloatLiteral(val.to_string());
            } else if let Token::Ident(name) = token {
                if self.peek(1).is_some_and(|t| t == &Token::LeftParen) {
                    self.consume()?;
                    self.consume()?;

                    let mut args: Vec<Expr> = Vec::new();

                    while self.peek(0).is_some_and(|t| t != &Token::RightParen) {
                        let arg = self.parse_expr(1, false)?;
                        args.push(arg);
                        if self.peek(0).unwrap() == &Token::Comma {
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
            } else if let Token::Minus = token {
                let Some(Token::FloatLiteral(val)) = self.peek(1) else {
                    return Err(error!(InvalidData, "Missing literal!"));
                };
                left = Expr::NegFloatLiteral(val.to_string());
                self.consume()?;
            } else if let Token::LeftParen = token {
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

            let op = match cur.unwrap() {
                &Token::Plus => Operator::Plus,
                &Token::Minus => Operator::Minus,
                &Token::Multi => Operator::Multi,
                &Token::Div => Operator::Div,
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
        self.consume()?;
        let mut block: Vec<Parsed> = Vec::new();

        while self.peek(0).is_some_and(|t| t != &Token::RightBrace) {
            let t = self.peek(0).unwrap();
            match t {
                Token::Ident(name) => {
                    if self.peek(1).is_some_and(|t| t == &Token::Equals) {
                        let out = self.parse_declaration(name.to_string())?;
                        block.push(out);
                    } else if self.peek(1).is_some_and(|t| t == &Token::LeftParen)
                        && self.line_contains_equals()
                    {
                        let out = self.parse_function_declaration(name.to_string())?;
                        block.push(out);
                    } else {
                        let out = self.parse_print()?;
                        block.push(out);
                    }
                }
                Token::Keyword(keyword) => {
                    if keyword.as_str() != "from" {
                        return Err(error!(Other, "Unexpected keyword: {}", keyword));
                    }
                    let out = self.parse_from_block()?;
                    block.push(out);
                }
                Token::FloatLiteral(_) => {
                    let out = self.parse_print()?;
                    block.push(out);
                }
                Token::Comment => {
                    while self.peek(0).is_some() && self.peek(0).unwrap() != &Token::Newline {
                        self.consume()?;
                    }
                }
                Token::Newline => {
                    self.consume()?;
                }
                token => todo!("Handle: {:?}", token),
            }
        }
        self.consume()?;

        Ok(Parsed::FromLoop(min, max, ident, block))
    }

    fn parse_declaration(&mut self, name: String) -> Result<Parsed> {
        self.consume()?;
        self.consume()?;
        let expr = self.parse_expr(1, false)?;
        Ok(Parsed::Declaration(name, expr))
    }

    fn parse_function_declaration(&mut self, name: String) -> Result<Parsed> {
        let mut parameters: Vec<String> = Vec::new();
        self.consume()?;
        self.consume()?;

        while self.peek(0).is_some_and(|t| t != &Token::RightParen) {
            if let Token::Ident(name) = self.peek(0).unwrap() {
                parameters.push(name.to_string());
            }
            self.consume()?;
        }
        self.consume()?;
        self.consume()?;
        let expr = self.parse_expr(1, true)?;
        Ok(Parsed::FunctionDecleration(
            name.to_string(),
            parameters,
            expr,
        ))
    }

    fn parse_print(&mut self) -> Result<Parsed> {
        let expr = self.parse_expr(1, false)?;

        Ok(Parsed::PrintExpr(expr))
    }

    fn line_contains_equals(&self) -> bool {
        for i in 0.. {
            if !self.peek(i).is_some() {
                return false;
            }

            let t = self.peek(i).unwrap();

            if t == &Token::Newline {
                return false;
            } else if t == &Token::Equals {
                return true;
            }
        }

        return false;
    }

    pub fn parse(&mut self) -> Result<Vec<Parsed>> {
        println!("Tokens: {:#?}", &self.tokens);
        while let Some(cur) = self.peek(0) {
            match &cur {
                Token::Ident(name) => {
                    if self.peek(1).is_some_and(|t| t == &Token::Equals) {
                        let out = self.parse_declaration(name.to_string())?;
                        self.parsed.push(out);
                    } else if self.peek(1).is_some_and(|t| t == &Token::LeftParen)
                        && self.line_contains_equals()
                    {
                        let out = self.parse_function_declaration(name.to_string())?;
                        self.parsed.push(out);
                    } else {
                        let out = self.parse_print()?;
                        self.parsed.push(out);
                    }
                }
                Token::Keyword(keyword) => {
                    if keyword.as_str() != "from" {
                        return Err(error!(Other, "Unexpected keyword: {}", keyword));
                    }
                    let out = self.parse_from_block()?;
                    self.parsed.push(out);
                }
                Token::FloatLiteral(_) => {
                    let out = self.parse_print()?;
                    self.parsed.push(out);
                }
                Token::Comment => {
                    while let Some(Token::Newline) = self.peek(0) {
                        self.consume()?;
                    }
                }
                Token::Newline => {
                    self.consume()?;
                }
                Token::RightParen => {
                    self.consume()?;
                }
                token => todo!("Handle: {:?}", token),
            };
        }

        Ok(self.parsed.to_vec())
    }
}
