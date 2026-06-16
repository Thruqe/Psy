mod token;
pub use token::Token;

pub struct Lexer {
    position: usize,
    chars: Vec<char>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        Lexer { position: 0, chars }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.position < self.chars.len() {
            let ch = self.chars[self.position];

            // Skip whitespace except newlines
            if ch.is_whitespace() && ch != '\n' {
                self.position += 1;
                continue;
            }

            // Handle newlines
            if ch == '\n' {
                tokens.push(Token::Newline);
                self.position += 1;
                continue;
            }

            // Single-line comments: //
            if ch == '/' && self.peek() == Some('/') {
                // Skip the rest of the line
                while self.position < self.chars.len() && self.chars[self.position] != '\n' {
                    self.position += 1;
                }
                // The newline will be added in the next iteration
                continue;
            }

            // Multi-line comments: /* */
            if ch == '/' && self.peek() == Some('*') {
                self.position += 2; // Skip /*
                while self.position < self.chars.len() {
                    if self.chars[self.position] == '*' && self.peek() == Some('/') {
                        self.position += 2; // Skip */
                        break;
                    }
                    self.position += 1;
                }
                continue;
            }

            // String literals
            if ch == '"' {
                let mut string = String::new();
                self.position += 1; // Skip opening quote

                while self.position < self.chars.len() && self.chars[self.position] != '"' {
                    string.push(self.chars[self.position]);
                    self.position += 1;
                }

                if self.position < self.chars.len() {
                    self.position += 1; // Skip closing quote
                }

                tokens.push(Token::StringLiteral(string));
                continue;
            }

            // Numbers
            if ch.is_ascii_digit()
                || (ch == '.' && self.peek().map_or(false, |c| c.is_ascii_digit()))
            {
                let mut number = String::new();
                let mut has_decimal = false;

                while self.position < self.chars.len() {
                    let c = self.chars[self.position];
                    if c == '.' && !has_decimal {
                        number.push(c);
                        has_decimal = true;
                        self.position += 1;
                    } else if c.is_ascii_digit() {
                        number.push(c);
                        self.position += 1;
                    } else {
                        break;
                    }
                }

                if let Ok(num) = number.parse::<f64>() {
                    tokens.push(Token::Number(num));
                }
                continue;
            }

            // Identifiers and keywords
            if ch.is_alphabetic() || ch == '_' {
                let mut ident = String::new();

                while self.position < self.chars.len() {
                    let c = self.chars[self.position];
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        self.position += 1;
                    } else {
                        break;
                    }
                }

                let token = match ident.as_str() {
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
                    "TRUE" => Token::Boolean(true),
                    "FALSE" => Token::Boolean(false),
                    _ => Token::Identifier(ident),
                };
                tokens.push(token);
                continue;
            }

            // Operators and punctuation
            match ch {
                '+' => {
                    tokens.push(Token::Plus);
                    self.position += 1;
                }
                '-' => {
                    tokens.push(Token::Minus);
                    self.position += 1;
                }
                '*' => {
                    tokens.push(Token::Star);
                    self.position += 1;
                }
                '/' => {
                    tokens.push(Token::Slash);
                    self.position += 1;
                }
                '%' => {
                    tokens.push(Token::Percent);
                    self.position += 1;
                }
                '^' => {
                    tokens.push(Token::Caret);
                    self.position += 1;
                }
                '=' => {
                    if self.peek() == Some('=') {
                        tokens.push(Token::Eq);
                        self.position += 2;
                    } else {
                        tokens.push(Token::Assign);
                        self.position += 1;
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        tokens.push(Token::LessEqual);
                        self.position += 2;
                    } else {
                        tokens.push(Token::LessThan);
                        self.position += 1;
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        tokens.push(Token::GreaterEqual);
                        self.position += 2;
                    } else {
                        tokens.push(Token::GreaterThan);
                        self.position += 1;
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        tokens.push(Token::NotEqual);
                        self.position += 2;
                    } else {
                        tokens.push(Token::Not);
                        self.position += 1;
                    }
                }
                '[' => {
                    tokens.push(Token::LeftBracket);
                    self.position += 1;
                }
                ']' => {
                    tokens.push(Token::RightBracket);
                    self.position += 1;
                }
                ',' => {
                    tokens.push(Token::Comma);
                    self.position += 1;
                }
                _ => {
                    // Unknown character - skip it
                    self.position += 1;
                }
            }
        }

        tokens.push(Token::EOF);
        tokens
    }

    fn peek(&self) -> Option<char> {
        if self.position + 1 < self.chars.len() {
            Some(self.chars[self.position + 1])
        } else {
            None
        }
    }
}
