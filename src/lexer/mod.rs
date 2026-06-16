pub mod token;

use self::token::Token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            tokens.push(token.clone());
            if token == Token::EOF {
                break;
            }
        }

        tokens
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Token::EOF;
        }

        let ch = self.input[self.position];

        match ch {
            '+' => {
                self.position += 1;
                self.col += 1;
                Token::Plus
            }
            '-' => {
                self.position += 1;
                self.col += 1;
                Token::Minus
            }
            '*' => {
                self.position += 1;
                self.col += 1;
                Token::Star
            }
            '/' => {
                self.position += 1;
                self.col += 1;
                Token::Slash
            }
            '%' => {
                self.position += 1;
                self.col += 1;
                Token::Percent
            }
            '^' => {
                self.position += 1;
                self.col += 1;
                Token::Caret
            }
            '=' => {
                self.position += 1;
                self.col += 1;
                if self.peek() == '=' {
                    self.position += 1;
                    self.col += 1;
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            '!' => {
                self.position += 1;
                self.col += 1;
                if self.peek() == '=' {
                    self.position += 1;
                    self.col += 1;
                    Token::NotEqual
                } else {
                    self.next_token()
                }
            }
            '>' => {
                self.position += 1;
                self.col += 1;
                if self.peek() == '=' {
                    self.position += 1;
                    self.col += 1;
                    Token::GreaterEqual
                } else {
                    Token::GreaterThan
                }
            }
            '<' => {
                self.position += 1;
                self.col += 1;
                if self.peek() == '=' {
                    self.position += 1;
                    self.col += 1;
                    Token::LessEqual
                } else {
                    Token::LessThan
                }
            }
            '[' => {
                self.position += 1;
                self.col += 1;
                Token::LeftBracket
            }
            ']' => {
                self.position += 1;
                self.col += 1;
                Token::RightBracket
            }
            ',' => {
                self.position += 1;
                self.col += 1;
                Token::Comma
            }
            '"' => self.read_string(),
            '0'..='9' => self.read_number(),
            'a'..='z' | 'A'..='Z' | '_' => {
                let token = self.read_identifier();
                if token == Token::Else && self.try_consume_if_after_else() {
                    Token::ElseIf
                } else {
                    token
                }
            }
            '\n' => {
                self.position += 1;
                self.line += 1;
                self.col = 1;
                Token::Newline
            }
            _ => {
                self.position += 1;
                self.col += 1;
                self.next_token()
            }
        }
    }

    fn read_string(&mut self) -> Token {
        self.position += 1;
        self.col += 1;
        let start = self.position;

        while self.position < self.input.len() && self.input[self.position] != '"' {
            self.position += 1;
        }

        let value: String = self.input[start..self.position].iter().collect();
        self.position += 1;
        self.col += value.len() as usize + 2;
        Token::StringLiteral(value)
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;

        while self.position < self.input.len()
            && (self.input[self.position].is_ascii_digit() || self.input[self.position] == '.')
        {
            self.position += 1;
        }

        let value: String = self.input[start..self.position].iter().collect();
        self.col += value.len();
        let num = value.parse().unwrap_or(0.0);
        Token::Number(num)
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;

        while self.position < self.input.len()
            && (self.input[self.position].is_ascii_alphanumeric()
                || self.input[self.position] == '_')
        {
            self.position += 1;
        }

        let value: String = self.input[start..self.position].iter().collect();
        self.col += value.len();

        match value.as_str() {
            "START" => Token::Start,
            "END" => Token::End,
            "INPUT" => Token::Input,
            "OUTPUT" => Token::Output,
            "IF" => Token::If,
            "THEN" => Token::Then,
            "ELSE" => Token::Else,
            "ELSEIF" => Token::ElseIf,
            "ENDIF" => Token::EndIf,
            "FOR" => Token::For,
            "TO" => Token::To,
            "ENDFOR" => Token::EndFor,
            "WHILE" => Token::While,
            "ENDWHILE" => Token::EndWhile,
            "DECLARE" => Token::Declare,
            "AND" => Token::And,
            "OR" => Token::Or,
            "NOT" => Token::Not,
            "True" => Token::Boolean(true),
            "False" => Token::Boolean(false),
            _ => Token::Identifier(value),
        }
    }

    /// Called right after lexing the word "ELSE". If, skipping only inline
    /// whitespace (not newlines), the next word is "IF" as a standalone token
    /// (not e.g. "IFX"), consume it and report success so the caller can
    /// fold ELSE + IF into a single ElseIf token. Otherwise roll back.
    fn try_consume_if_after_else(&mut self) -> bool {
        let saved_position = self.position;
        let saved_col = self.col;

        while self.position < self.input.len()
            && (self.input[self.position] == ' ' || self.input[self.position] == '\t')
        {
            self.position += 1;
            self.col += 1;
        }

        if self.position + 1 < self.input.len()
            && self.input[self.position] == 'I'
            && self.input[self.position + 1] == 'F'
        {
            let after = self.position + 2;
            let boundary_ok = after >= self.input.len()
                || !(self.input[after].is_ascii_alphanumeric() || self.input[after] == '_');

            if boundary_ok {
                self.position += 2;
                self.col += 2;
                return true;
            }
        }

        self.position = saved_position;
        self.col = saved_col;
        false
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len()
            && self.input[self.position].is_whitespace()
            && self.input[self.position] != '\n'
        {
            self.position += 1;
            self.col += 1;
        }
    }

    fn peek(&self) -> char {
        if self.position < self.input.len() {
            self.input[self.position]
        } else {
            '\0'
        }
    }
}
