use crate::util::error;
use std::{
    fmt::Display,
    io::{Error, ErrorKind, Result},
};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Ident(String),
    FloatLiteral(String),
    Comment,
    Unknown(char),
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

#[derive(Debug, Clone)]
/// The location of a [`Token`] in form (file name, column, row).
pub struct TokenLocation(pub String, pub u32, pub u32);

#[derive(Debug, Clone)]
pub struct Token(pub TokenType, pub TokenLocation);

impl Display for TokenLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.0, self.2, self.1)?;

        Ok(())
    }
}

pub struct Lexer {
    file_path: String,
    content: String,
    tokens: Vec<Token>,
    index: usize,
}

impl Lexer {
    pub fn new(file_path: String, content: String) -> Self {
        Self {
            file_path,
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

    fn parse_text(&mut self, col: u32, row: u32) -> Result<()> {
        let mut buf = String::new();
        buf.push(self.consume()?);

        while self.peek(0).is_some_and(|c| c.is_ascii_alphabetic()) {
            buf.push(self.consume()?);
        }

        match buf.as_str() {
            "from" | "to" | "as" => self.tokens.push(Token(
                TokenType::Keyword(buf),
                TokenLocation(self.file_path.clone(), col, row),
            )),
            _ => self.tokens.push(Token(
                TokenType::Ident(buf),
                TokenLocation(self.file_path.clone(), col, row),
            )),
        }

        Ok(())
    }

    fn parse_float(&mut self, row: u32, col: u32) -> Result<()> {
        let mut buf = String::new();
        let mut period = false;

        let first = self.consume()?;
        if first == '.' {
            period = true;
            buf.push_str("0.");
        } else {
            buf.push(first);
        }

        while self
            .peek(0)
            .is_some_and(|c| c.is_digit(10) || c == '.' || c == '_')
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

        self.tokens.push(Token(
            TokenType::FloatLiteral(buf),
            TokenLocation(self.file_path.clone(), col, row),
        ));

        Ok(())
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut col = 0u32;
        let mut line = 0u32;
        while self.peek(0).is_some() {
            let c = self.peek(0).unwrap();
            if c == '\n' {
                self.tokens.push(Token(
                    TokenType::Newline,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                line += 1;
                col = 0;
                self.consume()?;
            } else if c.is_whitespace() {
                self.consume()?;
            } else if c.is_ascii_alphabetic() {
                self.parse_text(col + 1, line + 1)?;
            } else if c == '.' || c.is_digit(10) {
                self.parse_float(col + 1, line + 1)?;
            } else if c == '=' {
                self.tokens.push(Token(
                    TokenType::Equals,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else if c == '+' {
                self.tokens.push(Token(
                    TokenType::Plus,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else if c == '-' {
                self.tokens.push(Token(
                    TokenType::Minus,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else if c == '*' {
                self.tokens.push(Token(
                    TokenType::Multi,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else if c == ',' {
                self.tokens.push(Token(
                    TokenType::Comment,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else if c == '/' {
                self.tokens.push(Token(
                    TokenType::Div,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else if c == '#' {
                self.tokens.push(Token(
                    TokenType::Comment,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else if c == '(' {
                self.tokens.push(Token(
                    TokenType::LeftParen,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else if c == ')' {
                self.tokens.push(Token(
                    TokenType::RightParen,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else if c == '{' {
                self.tokens.push(Token(
                    TokenType::LeftBrace,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else if c == '}' {
                self.tokens.push(Token(
                    TokenType::RightBrace,
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            } else {
                self.tokens.push(Token(
                    TokenType::Unknown(c),
                    TokenLocation(self.file_path.clone(), col + 1, line + 1),
                ));
                self.consume()?;
            }

            if c != '\n' {
                col += 1;
            }
        }

        Ok(self.tokens.to_vec())
    }
}
