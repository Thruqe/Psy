mod token;
pub use token::{PositionedToken, Token};

pub struct Lexer {
    position: usize,
    chars: Vec<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        Lexer {
            position: 0,
            chars,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<PositionedToken> {
        let mut tokens = Vec::new();

        while self.position < self.chars.len() {
            let ch = self.chars[self.position];

            // Skip whitespace except newlines
            if ch.is_whitespace() && ch != '\n' {
                self.advance_char();
                continue;
            }

            // Handle newlines
            if ch == '\n' {
                let (line, column) = (self.line, self.column);
                self.advance_char();
                tokens.push(PositionedToken {
                    token: Token::Newline,
                    line,
                    column,
                });
                continue;
            }

            // Single-line comments: //
            if ch == '/' && self.peek() == Some('/') {
                while self.position < self.chars.len() && self.chars[self.position] != '\n' {
                    self.advance_char();
                }
                continue;
            }

            // Multi-line comments: /* */
            if ch == '/' && self.peek() == Some('*') {
                self.advance_char();
                self.advance_char();
                while self.position < self.chars.len() {
                    if self.chars[self.position] == '*' && self.peek() == Some('/') {
                        self.advance_char();
                        self.advance_char();
                        break;
                    }
                    self.advance_char();
                }
                continue;
            }

            // Extra Single-line comments: #

            if ch == '#' {
                while self.position < self.chars.len() && self.chars[self.position] != '\n' {
                    self.advance_char();
                }
                continue;
            }

            // String literals
            if ch == '"' {
                let (line, column) = (self.line, self.column);
                let mut string = String::new();
                self.advance_char(); // Skip opening quote

                while self.position < self.chars.len() && self.chars[self.position] != '"' {
                    string.push(self.chars[self.position]);
                    self.advance_char();
                }

                if self.position < self.chars.len() {
                    self.advance_char(); // Skip closing quote
                }

                tokens.push(PositionedToken {
                    token: Token::StringLiteral(string),
                    line,
                    column,
                });
                continue;
            }

            // Numbers
            if ch.is_ascii_digit()
                || (ch == '.' && self.peek().map_or(false, |c| c.is_ascii_digit()))
            {
                let (line, column) = (self.line, self.column);
                let mut number = String::new();
                let mut has_decimal = false;

                while self.position < self.chars.len() {
                    let c = self.chars[self.position];
                    if c == '.' && !has_decimal {
                        number.push(c);
                        has_decimal = true;
                        self.advance_char();
                    } else if c.is_ascii_digit() {
                        number.push(c);
                        self.advance_char();
                    } else {
                        break;
                    }
                }

                if let Ok(num) = number.parse::<f64>() {
                    tokens.push(PositionedToken {
                        token: Token::Number(num),
                        line,
                        column,
                    });
                }
                continue;
            }

            // Identifiers and keywords
            if ch.is_alphabetic() || ch == '_' {
                let (line, column) = (self.line, self.column);
                let mut ident = String::new();

                while self.position < self.chars.len() {
                    let c = self.chars[self.position];
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        self.advance_char();
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
                    "FUNCTION" => Token::Function,
                    "ENDFUNCTION" => Token::EndFunction,
                    "RETURN" => Token::Return,
                    "IMPORT" => Token::Import,
                    "PUB" => Token::Pub,
                    "CONST" => Token::Const,
                    "STATIC" => Token::Static,
                    _ => Token::Identifier(ident),
                };
                tokens.push(PositionedToken {
                    token,
                    line,
                    column,
                });
                continue;
            }

            // Operators and punctuation
            let (line, column) = (self.line, self.column);
            match ch {
                '+' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::Plus,
                        line,
                        column,
                    });
                }
                '-' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::Minus,
                        line,
                        column,
                    });
                }
                '*' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::Star,
                        line,
                        column,
                    });
                }
                '/' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::Slash,
                        line,
                        column,
                    });
                }
                '%' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::Percent,
                        line,
                        column,
                    });
                }
                '^' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::Caret,
                        line,
                        column,
                    });
                }
                '=' => {
                    if self.peek() == Some('=') {
                        self.advance_char();
                        self.advance_char();
                        tokens.push(PositionedToken {
                            token: Token::Eq,
                            line,
                            column,
                        });
                    } else {
                        self.advance_char();
                        tokens.push(PositionedToken {
                            token: Token::Assign,
                            line,
                            column,
                        });
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        self.advance_char();
                        self.advance_char();
                        tokens.push(PositionedToken {
                            token: Token::LessEqual,
                            line,
                            column,
                        });
                    } else {
                        self.advance_char();
                        tokens.push(PositionedToken {
                            token: Token::LessThan,
                            line,
                            column,
                        });
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.advance_char();
                        self.advance_char();
                        tokens.push(PositionedToken {
                            token: Token::GreaterEqual,
                            line,
                            column,
                        });
                    } else {
                        self.advance_char();
                        tokens.push(PositionedToken {
                            token: Token::GreaterThan,
                            line,
                            column,
                        });
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance_char();
                        self.advance_char();
                        tokens.push(PositionedToken {
                            token: Token::NotEqual,
                            line,
                            column,
                        });
                    } else {
                        self.advance_char();
                        tokens.push(PositionedToken {
                            token: Token::Not,
                            line,
                            column,
                        });
                    }
                }
                '[' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::LeftBracket,
                        line,
                        column,
                    });
                }
                ']' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::RightBracket,
                        line,
                        column,
                    });
                }
                ',' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::Comma,
                        line,
                        column,
                    });
                }
                '(' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::LeftParen,
                        line,
                        column,
                    });
                }
                ')' => {
                    self.advance_char();
                    tokens.push(PositionedToken {
                        token: Token::RightParen,
                        line,
                        column,
                    });
                }
                _ => {
                    // Unknown character - skip it
                    self.advance_char();
                }
            }
        }

        tokens.push(PositionedToken {
            token: Token::EOF,
            line: self.line,
            column: self.column,
        });
        tokens
    }

    fn peek(&self) -> Option<char> {
        if self.position + 1 < self.chars.len() {
            Some(self.chars[self.position + 1])
        } else {
            None
        }
    }

    /// Advances position by one character, updating line/column tracking.
    fn advance_char(&mut self) {
        if self.position < self.chars.len() {
            if self.chars[self.position] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }
}
