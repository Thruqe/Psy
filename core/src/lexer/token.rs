use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct PositionedToken {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

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
    Function,
    EndFunction,
    Return,

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

    //
    LeftParen,
    RightParen,

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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Start => write!(f, "START"),
            Token::End => write!(f, "END"),
            Token::Input => write!(f, "INPUT"),
            Token::Output => write!(f, "OUTPUT"),
            Token::If => write!(f, "IF"),
            Token::Then => write!(f, "THEN"),
            Token::Else => write!(f, "ELSE"),
            Token::ElseIf => write!(f, "ELSEIF"),
            Token::EndIf => write!(f, "ENDIF"),
            Token::For => write!(f, "FOR"),
            Token::To => write!(f, "TO"),
            Token::EndFor => write!(f, "ENDFOR"),
            Token::While => write!(f, "WHILE"),
            Token::EndWhile => write!(f, "ENDWHILE"),
            Token::Declare => write!(f, "DECLARE"),
            Token::Function => write!(f, "FUNCTION"),
            Token::EndFunction => write!(f, "ENDFUNCTION"),
            Token::Return => write!(f, "RETURN"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Caret => write!(f, "^"),
            Token::Eq => write!(f, "=="),
            Token::Assign => write!(f, "="),
            Token::LessThan => write!(f, "<"),
            Token::GreaterThan => write!(f, ">"),
            Token::LessEqual => write!(f, "<="),
            Token::GreaterEqual => write!(f, ">="),
            Token::NotEqual => write!(f, "!="),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::And => write!(f, "AND"),
            Token::Or => write!(f, "OR"),
            Token::Not => write!(f, "NOT"),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::Number(n) => write!(f, "{}", n),
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),
            Token::Boolean(b) => write!(f, "{}", b),
            Token::Newline => write!(f, "\\n"),
            Token::EOF => write!(f, "EOF"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
        }
    }
}
