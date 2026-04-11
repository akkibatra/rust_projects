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