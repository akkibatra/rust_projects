use std::collections::HashMap;
use crate::lexer::Token;
use crate::value::JsonValue;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String, usize),
    UnexpectedEOF,
    InvalidNumber(String),
    InvalidString(String)
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(msg, pos) => write!(f, "Unexpected token at pos {}: {}", pos, msg),
            ParseError::UnexpectedEOF => write!(f, "Unexpected end of file"),
            ParseError::InvalidNumber(n) => write!(f, "Invalid number: {}", n),
            ParseError::InvalidString(s) => write!(f, "Invalid string: {}", s),
        }
    }
}

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    cursor: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, cursor: 0}
    }

    pub fn parse(&mut self) -> Result<JsonValue<'a>, ParseError> {
        let token = self.advance()
            .cloned()
            .ok_or(ParseError::UnexpectedEOF)?;
        let pos = self.cursor;

        match token {
            Token::Null => Ok(JsonValue::Null),
            Token::Boolean(b) => Ok(JsonValue::Boolean(b)),
            Token::Number(f) => Ok(JsonValue::Number(f)),
            Token::String(s) => Ok(JsonValue::String(s)),
            Token::LeftBracket => self.parse_array(),
            Token::LeftBrace => self.parse_object(),
            _ => Err(ParseError::UnexpectedToken(format!("{:?}", token), pos))
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue<'a>, ParseError> {
        let mut array = Vec::new();

        if let Some(Token::RightBracket) = self.peek() {
            self.advance();
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse()?;
            array.push(value);

            match self.advance() {
                Some(Token::Comma) => continue,
                Some(Token::RightBracket) => break,
                Some(t) => return Err(ParseError::UnexpectedToken(format!("{:?}", t), self.cursor)),
                None => return Err(ParseError::UnexpectedEOF),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue<'a>, ParseError> {
        let mut object = HashMap::new();

        if let Some(Token::RightBrace) = self.peek() {
            self.advance();
            return Ok(JsonValue::Object(object));
        }

        loop {
            // key
            let key = match self.advance() {
                Some(Token::String(s)) => *s,
                Some(t) => return Err(ParseError::UnexpectedToken(format!("{:?}", t), self.cursor)),
                None => return Err(ParseError::UnexpectedEOF),
            };

            // colon
            if !matches!(self.advance(), Some(Token::Colon)) {
                return Err(ParseError::UnexpectedToken(format!("{:?}", self.advance()), self.cursor));
            }

            // JsonValue
            let value = self.parse()?;
            object.insert(key, value);

            // , or }
            match self.advance() {
                Some(Token::Comma) => continue,
                Some(Token::RightBrace) => break,
                Some(t) => return Err(ParseError::UnexpectedToken(format!("{:?}", t), self.cursor)),
                None => return Err(ParseError::UnexpectedEOF),
            }
        }

        Ok(JsonValue::Object(object))
    }

    fn peek(&mut self) -> Option<&Token<'a>> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) -> Option<&Token<'a>> {
        let token = self.tokens.get(self.cursor);
        self.cursor += 1;
        token
    }
}