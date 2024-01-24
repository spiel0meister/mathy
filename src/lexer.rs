use crate::util::error;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    FloatLiteral(String),
    Comment,
    Unkown(char),
    Newline,
    Equals,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Keyword(String),
    Comma,
    Plus,
    Minus,
    Multi,
    Div,
}

pub struct Lexer {
    content: String,
    tokens: Vec<Token>,
    index: usize,
}

impl Lexer {
    pub fn new(content: String) -> Self {
        Self {
            content,
            tokens: Vec::new(),
            index: 0,
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        if self.index + offset < self.content.len() {
            self.content.chars().nth(self.index + offset)
        } else {
            None
        }
    }

    fn consume(&mut self) -> Result<char> {
        let cur = self
            .content
            .chars()
            .nth(self.index)
            .ok_or(Error::new(ErrorKind::UnexpectedEof, "End of code!"))?;
        self.index += 1;
        Ok(cur)
    }

    fn parse_text(&mut self) -> Result<()> {
        let mut buf = String::new();
        buf.push(self.consume()?);

        while self.peek(0).is_some() && self.peek(0).unwrap().is_ascii_alphabetic() {
            buf.push(self.consume()?);
        }

        match buf.as_str() {
            "from" | "to" | "as" => self.tokens.push(Token::Keyword(buf)),
            _ => self.tokens.push(Token::Ident(buf)),
        }

        Ok(())
    }

    fn parse_float(&mut self) -> Result<()> {
        let mut buf = String::new();
        let mut period = false;

        let first = self.consume()?;
        if first == '.' {
            period = true;
            buf.push_str("0.");
        } else {
            buf.push(first);
        }

        while self.peek(0).is_some()
            && (self.peek(0).unwrap().is_digit(10)
                || self.peek(0).unwrap() == '.'
                || self.peek(0).unwrap() == '_')
        {
            if self.peek(0).unwrap() == '_' {
                self.consume()?;
                continue;
            }

            if self.peek(0).unwrap() == '.' {
                if period {
                    return Err(error!(Other, "Multiple periods!"));
                } else {
                    period = true;
                }
            }
            buf.push(self.consume()?);
        }

        if !buf.contains('.') {
            buf.push_str(".0");
        }

        self.tokens.push(Token::FloatLiteral(buf));

        Ok(())
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        while self.peek(0).is_some() {
            let c = self.peek(0).unwrap();
            if c == '\n' {
                self.tokens.push(Token::Newline);
                self.consume()?;
            } else if c.is_whitespace() {
                self.consume()?;
            } else if c.is_ascii_alphabetic() {
                self.parse_text()?;
            } else if c == '.' || c.is_digit(10) {
                self.parse_float()?;
            } else if c == '=' {
                self.tokens.push(Token::Equals);
                self.consume()?;
            } else if c == '+' {
                self.tokens.push(Token::Plus);
                self.consume()?;
            } else if c == '-' {
                self.tokens.push(Token::Minus);
                self.consume()?;
            } else if c == '*' {
                self.tokens.push(Token::Multi);
                self.consume()?;
            } else if c == ',' {
                self.tokens.push(Token::Comma);
                self.consume()?;
            } else if c == '/' {
                self.tokens.push(Token::Div);
                self.consume()?;
            } else if c == '#' {
                self.tokens.push(Token::Comment);
                self.consume()?;
            } else if c == '(' {
                self.tokens.push(Token::LeftParen);
                self.consume()?;
            } else if c == ')' {
                self.tokens.push(Token::RightParen);
                self.consume()?;
            } else if c == '{' {
                self.tokens.push(Token::LeftBrace);
                self.consume()?;
            } else if c == '}' {
                self.tokens.push(Token::RightBrace);
                self.consume()?;
            } else {
                self.tokens.push(Token::Unkown(c));
                self.consume()?;
            }
        }

        Ok(self.tokens.to_vec())
    }
}
