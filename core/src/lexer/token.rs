#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Start,
    End,
    Input,
    Output,
    If,
    Then,
    Else,
    ElseIf,
    EndIf,
    For,
    To,
    EndFor,
    While,
    EndWhile,
    Declare,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Eq,
    Assign,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    NotEqual,

    // Array brackets
    LeftBracket,
    RightBracket,

    // Punctuation
    Comma,

    // Logical
    And,
    Or,
    Not,

    // Values
    Identifier(String),
    Number(f64),
    StringLiteral(String),
    Boolean(bool),

    // Structure
    Newline,
    EOF,
}

impl Token {
    pub fn to_string(&self) -> String {
        match self {
            Token::Start => "START".to_string(),
            Token::End => "END".to_string(),
            Token::Input => "INPUT".to_string(),
            Token::Output => "OUTPUT".to_string(),
            Token::If => "IF".to_string(),
            Token::Then => "THEN".to_string(),
            Token::Else => "ELSE".to_string(),
            Token::ElseIf => "ELSEIF".to_string(),
            Token::EndIf => "ENDIF".to_string(),
            Token::For => "FOR".to_string(),
            Token::To => "TO".to_string(),
            Token::EndFor => "ENDFOR".to_string(),
            Token::While => "WHILE".to_string(),
            Token::EndWhile => "ENDWHILE".to_string(),
            Token::Declare => "DECLARE".to_string(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Star => "*".to_string(),
            Token::Slash => "/".to_string(),
            Token::Percent => "%".to_string(),
            Token::Caret => "^".to_string(),
            Token::Eq => "==".to_string(),
            Token::Assign => "=".to_string(),
            Token::LessThan => "<".to_string(),
            Token::GreaterThan => ">".to_string(),
            Token::LessEqual => "<=".to_string(),
            Token::GreaterEqual => ">=".to_string(),
            Token::NotEqual => "!=".to_string(),
            Token::LeftBracket => "[".to_string(),
            Token::RightBracket => "]".to_string(),
            Token::Comma => ",".to_string(),
            Token::And => "AND".to_string(),
            Token::Or => "OR".to_string(),
            Token::Not => "NOT".to_string(),
            Token::Identifier(s) => s.clone(),
            Token::Number(n) => n.to_string(),
            Token::StringLiteral(s) => format!("\"{}\"", s),
            Token::Boolean(b) => b.to_string(),
            Token::Newline => "\\n".to_string(),
            Token::EOF => "EOF".to_string(),
        }
    }
}
