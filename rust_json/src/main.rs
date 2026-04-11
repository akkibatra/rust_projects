use core::f64;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue<'a> {
    Null,
    Boolean(bool),
    Number(f64),
    String(&'a str),
    Array(Vec<JsonValue<'a>>),
    Object(std::collections::HashMap<&'a str, JsonValue<'a>>),
}

impl<'a> JsonValue<'a> {
    pub fn stringify(&self) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Boolean(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", s),
            JsonValue::Array(arr) => {
                let elements: Vec<String> = arr.iter()
                    .map(|v| v.stringify())
                    .collect();

                format!("[{}]", elements.join(","))
            },
            JsonValue::Object(obj) => {
                let pairs: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, v.stringify()))
                    .collect();

                format!("{{{}}}", pairs.join(","))
            },
        }
    }

    pub fn to_json_string(&self) -> String {
        let mut buf = String::new();
        self.serialize(&mut buf);
        buf
    }

    fn serialize(&self, buf: &mut String) {
        match self {
            JsonValue::Null => buf.push_str("null"),
            JsonValue::Boolean(b) => buf.push_str(&b.to_string()),
            JsonValue::Number(n) => buf.push_str(&n.to_string()),
            JsonValue::String(s) => {
                buf.push('"');
                buf.push_str(s);
                buf.push('"');
            },
            JsonValue::Array(arr) => {
                buf.push('[');
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 { buf.push(',');}
                    val.serialize(buf);
                }
                buf.push(']');
            },
            JsonValue::Object(obj) => {
                buf.push('{');
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 { buf.push(',');}
                    buf.push('"');
                    buf.push_str(key);
                    buf.push_str("\":");
                    val.serialize(buf);
                }
                buf.push('}');
            },
        }
    }

    pub fn to_pretty_string(&self) -> String {
        let mut buf = String::new();
        self.serialize_pretty(&mut buf, 0);
        buf
    }

    fn serialize_pretty(&self, buf: &mut String, depth: usize) {
        let indent = "  ".repeat(depth);
        let child_indent = "  ".repeat(depth + 1);

        match self {
            JsonValue::Null => buf.push_str("null"),
            JsonValue::Boolean(b) => buf.push_str(&b.to_string()),
            JsonValue::Number(n) => buf.push_str(&n.to_string()),
            JsonValue::String(s) => {
                buf.push('"');
                buf.push_str(s);
                buf.push('"');
            },
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    buf.push_str("[]");
                } else {
                    buf.push_str("[\n");
                    for (i, val) in arr.iter().enumerate() {
                        if i > 0 { buf.push_str(",\n");}
                        buf.push_str(&child_indent);
                        val.serialize_pretty(buf, depth + 1);
                    }
                    buf.push_str("\n");
                    buf.push_str(&indent);
                    buf.push(']');
                }
            },
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    buf.push_str("{}");
                } else {
                    buf.push_str("{\n");
                    for (i, (key, val)) in obj.iter().enumerate() {
                        if i > 0 { buf.push_str(",\n");}
                        buf.push_str(&child_indent);
                        buf.push('"');
                        buf.push_str(key);
                        buf.push_str("\": ");
                        val.serialize_pretty(buf, depth + 1);
                    }
                    buf.push_str("\n");
                    buf.push_str(&indent);
                    buf.push('}');
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token<'a> {
    LeftBrace, // {
    RightBrace, // }
    LeftBracket, // [
    RightBracket, // ]
    Colon, // :
    Comma, // ,
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}

pub struct Lexer<'a> {
    input: &'a str,
    cursor: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            cursor: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();

        while let Some(token) = self.new_token() {
            tokens.push(token);
        }

        tokens
    }

    fn peek(&mut self) -> Option<char> {
        self.input.get(self.cursor..)?.chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek();
        self.cursor += 1;
        c
    }

    fn new_token(&mut self) -> Option<Token<'a>> {
        self.consume_whitespace();

        let c = self.next_char()?;

        match c {
            '{' => Some(Token::LeftBrace),
            '}' => Some(Token::RightBrace),
            '[' => Some(Token::LeftBracket),
            ']' => Some(Token::RightBracket),
            ':' => Some(Token::Colon),
            ',' => Some(Token::Comma),
            't' => self.lex_keyword("true", Token::Boolean(true)),
            'f' => self.lex_keyword("false", Token::Boolean(false)),
            'n' => self.lex_keyword("null", Token::Null),
            '0'..='9' | '-' => self.lex_number(c),
            '"' => self.lex_string(),
            _ => None
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() { self.next_char(); }
            else { break; }
        }
    }

    fn lex_keyword(&mut self, keyword: &str, token: Token<'a>) -> Option<Token<'a>> {
        for expected in keyword.chars().skip(1) {
            if self.next_char()? != expected {
                return None; 
            }
        }
        Some(token)
    }

    fn lex_number(&mut self, first_char: char) -> Option<Token<'a>> {
        let mut number_str = String::new();
        number_str.push(first_char);

        while let Some(c) = self.peek() {
            match c {
                '0'..='9' | '.' | 'e' | 'E' | '+' | '-' => {
                    number_str.push(self.next_char().unwrap());
                }
                _ => break,
            }
        }

        match number_str.parse::<f64>() {
            Ok(num) => Some(Token::Number(num)),
            Err(_) => None,
        }
    }

    fn lex_string(&mut self) -> Option<Token<'a>> {
        let start = self.cursor;

        while let Some(c) = self.peek() {
            if c == '"' {
                let end = self.cursor;
                self.next_char();

                return Some(Token::String(&self.input[start..end]));
            }
            self.next_char();
        }

        None
    }
}

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

fn main() -> Result<(), ParseError> {
    let input = r#"{"user":"Akshay","projects":["kv-store","json-parser"],"active":true}"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let json = parser.parse()?;
    
    println!("--- Compact ---");
    println!("{}", json.to_json_string());
    
    println!("\n--- Pretty ---");
    println!("{}", json.to_pretty_string());

    Ok(())
}
