use crate::util::error;
use std::{
    fmt::Display,
    io::{Error, ErrorKind, Result},
};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    /// Represents an identifier.
    Ident(String),
    /// Represents a float.
    FloatLiteral(String),
    /// Represents the '#' character.
    Comment,
    /// Represents an unknown character.
    Unknown(char),
    /// Represents a newline character.
    Newline,
    /// Represents the '=' character.
    Equals,
    /// Represents the '(' character.
    LeftParen,
    /// Represents the ')' character.
    RightParen,
    /// Represents the '{' character.
    LeftBrace,
    /// Represents the '}' character.
    RightBrace,
    /// Represents the '[' character.
    LeftBracket,
    /// Represents the ']' character.
    RightBracket,
    /// Represents a keyword.
    Keyword(String),
    /// Represents the ',' character.
    Comma,
    /// Represents the '+' character.
    Plus,
    /// Represents the '-' character.
    Minus,
    /// Represents the '*' character.
    Multi,
    /// Represents the '/' character.
    Div,
    /// Represents the '^' character.
    Circumflex,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Multi => "*",
            Self::Div => "/",
            Self::Comma => ",",
            Self::LeftParen => "(",
            Self::LeftBrace => "{",
            Self::LeftBracket => "[",
            Self::RightParen => ")",
            Self::RightBrace => "}",
            Self::RightBracket => "]",
            Self::Equals => "=",
            Self::Comment => "#",
            Self::Newline => r#"\n"#,
            Self::FloatLiteral(literal) => literal,
            Self::Ident(name) => name,
            Self::Keyword(keyword) => keyword,
            Self::Circumflex => "^",
            Self::Unknown(c) => {
                return write!(f, "{:?}", c);
            }
        };

        write!(f, "{:?}", c)
    }
}

impl From<char> for TokenType {
    fn from(value: char) -> Self {
        match value {
            '+' => Self::Plus,
            '-' => Self::Minus,
            '*' => Self::Multi,
            '/' => Self::Div,
            ',' => Self::Comma,
            '(' => Self::LeftParen,
            '{' => Self::LeftBrace,
            '[' => Self::LeftBracket,
            ')' => Self::RightParen,
            '}' => Self::RightBrace,
            ']' => Self::RightBracket,
            '=' => Self::Equals,
            '#' => Self::Comment,
            '\n' => Self::Newline,
            '^' => Self::Circumflex,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone)]
/// The location of a [`Token`] in the form (file name, column, row).
pub struct TokenLocation(pub String, pub u32, pub u32);

#[derive(Debug, Clone)]
pub struct Token(pub TokenType, pub TokenLocation);

impl Token {
    pub fn exclude_loc(self) -> TokenType {
        self.0
    }
}

impl Display for TokenLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.0, self.2, self.1)?;

        Ok(())
    }
}

macro_rules! token {
    ($type:expr, $file:expr, $col:expr, $row:expr) => {
        Token($type, TokenLocation($file, $col, $row))
    };
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
            .ok_or(Error::new(ErrorKind::UnexpectedEof, "Unexpected EOF"))?;
        self.index += 1;
        Ok(cur)
    }

    fn parse_text(&mut self, col: u32, row: u32) -> Result<u32> {
        let mut col_delta = 0u32;
        let mut buf = String::new();
        buf.push(self.consume()?);

        while self
            .peek(0)
            .is_some_and(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_')
        {
            col_delta += 1;
            buf.push(self.consume()?);
        }

        match buf.as_str() {
            "from" | "to" | "as" | "with" | "step" | "for" | "in" => self.tokens.push(token!(
                TokenType::Keyword(buf),
                self.file_path.clone(),
                col,
                row
            )),
            _ => self.tokens.push(token!(
                TokenType::Ident(buf),
                self.file_path.clone(),
                col,
                row
            )),
        }

        Ok(col_delta)
    }

    fn parse_float(&mut self, row: u32, col: u32) -> Result<u32> {
        let mut col_delta = 0u32;
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
                col_delta += 1;
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
            col_delta += 1;
        }

        if !buf.contains('.') {
            buf.push_str(".0");
        }

        self.tokens.push(token!(
            TokenType::FloatLiteral(buf),
            self.file_path.clone(),
            col,
            row
        ));

        Ok(col_delta)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut col = 1u32;
        let mut line = 1u32;
        while self.peek(0).is_some() {
            let c = self.peek(0).unwrap();
            if c == '\n' {
                self.tokens.push(token!(
                    TokenType::Newline,
                    self.file_path.clone(),
                    col,
                    line
                ));
                line += 1;
                col = 0;
                self.consume()?;
            } else if c.is_whitespace() {
                self.consume()?;
            } else if c.is_ascii_alphabetic() || c == '_' {
                col += self.parse_text(line, col)?;
            } else if c == '.' || c.is_digit(10) {
                col += self.parse_float(line, col)?;
            } else {
                self.tokens.push(token!(
                    TokenType::from(c),
                    self.file_path.clone(),
                    col,
                    line
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
