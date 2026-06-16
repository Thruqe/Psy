pub mod ast;

use self::ast::{Expression, Operator, OutputValue, Statement, UnaryOperator};
use crate::lexer::Token; 

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        self.skip_newlines();

        if self.check(Token::Start) {
            self.advance();
            self.skip_newlines();
        } else {
            eprintln!("Warning: Program should start with START");
        }

        let statements = self.parse_block_statements(&[Token::End]);

        if self.check(Token::End) {
            self.advance();
        } else {
            eprintln!("Warning: Program should end with END");
        }

        statements
    }

    /// Parses statements until EOF or one of `stop_tokens` is reached.
    /// If an individual statement fails to parse, the error is reported
    /// and parsing recovers locally (skipping just that statement) instead
    /// of unwinding out of the surrounding block.
    fn parse_block_statements(&mut self, stop_tokens: &[Token]) -> Vec<Statement> {
        let mut statements = Vec::new();

        loop {
            self.skip_newlines();

            if self.is_at_end() || self.at_any(stop_tokens) {
                break;
            }

            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    self.recover_to_boundary(stop_tokens);
                }
            }
        }

        statements
    }

    /// Skips tokens until a newline, one of `stop_tokens`, or EOF is found,
    /// then consumes any trailing newlines. Used to resynchronize after a
    /// statement-level parse error without losing track of the enclosing
    /// block's terminator (ELSE/ELSEIF/ENDIF/ENDFOR/ENDWHILE/END).
    fn recover_to_boundary(&mut self, stop_tokens: &[Token]) {
        while !self.is_at_end() && !self.check(Token::Newline) && !self.at_any(stop_tokens) {
            self.advance();
        }
        self.skip_newlines();
    }

    fn at_any(&self, tokens: &[Token]) -> bool {
        let current = self.current();
        tokens.iter().any(|t| t == &current)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        self.skip_newlines();

        if self.is_at_end() {
            return Err("Unexpected end of file".to_string());
        }

        if self.check(Token::Input) {
            self.parse_input()
        } else if self.check(Token::Output) {
            self.parse_output()
        } else if self.check(Token::If) {
            self.parse_if()
        } else if self.check(Token::For) {
            self.parse_for()
        } else if self.check(Token::While) {
            self.parse_while()
        } else if self.check(Token::Declare) {
            self.parse_declare()
        } else if self.peek_is_identifier() && self.peek_next_is_assign() {
            self.parse_assign()
        } else if self.peek_is_array_access() {
            self.parse_array_assign()
        } else {
            let token = self.current();
            self.advance();
            Err(format!(
                "Unknown statement starting with: {}",
                token.to_string()
            ))
        }
    }

    fn parse_input(&mut self) -> Result<Statement, String> {
        self.advance();
        self.skip_newlines();

        let mut variables = Vec::new();

        while !self.is_at_end() && !self.check(Token::Newline) {
            if let Token::Identifier(name) = self.current() {
                variables.push(name.clone());
                self.advance();
                if self.check(Token::Comma) {
                    self.advance();
                }
            } else {
                return Err("Expected identifier after INPUT".to_string());
            }
        }

        self.skip_newlines();
        Ok(Statement::Input { variables })
    }

    fn parse_output(&mut self) -> Result<Statement, String> {
        self.advance();
        self.skip_newlines();

        let mut values = Vec::new();

        while !self.is_at_end() && !self.check(Token::Newline) {
            let value = if let Token::StringLiteral(s) = self.current() {
                self.advance();
                OutputValue::StringLiteral(s.clone())
            } else {
                let expr = self.parse_expression()?;
                OutputValue::Expression(expr)
            };
            values.push(value);

            if self.check(Token::Comma) {
                self.advance();
            }
        }

        self.skip_newlines();
        Ok(Statement::Output { values })
    }

    fn parse_assign(&mut self) -> Result<Statement, String> {
        let var_name = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err("Expected identifier".to_string());
        };

        self.advance();
        self.skip_newlines();

        if !self.check(Token::Assign) {
            return Err("Expected = for assignment".to_string());
        }
        self.advance();
        self.skip_newlines();

        let expr = self.parse_expression()?;

        self.skip_newlines();
        Ok(Statement::Assign {
            variable: var_name,
            expression: expr,
        })
    }

    fn parse_array_assign(&mut self) -> Result<Statement, String> {
        let name = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err("Expected array name".to_string());
        };
        self.advance();

        if !self.check(Token::LeftBracket) {
            return Err("Expected [ for array access".to_string());
        }
        self.advance();

        let index = self.parse_expression()?;

        if !self.check(Token::RightBracket) {
            return Err("Expected ] for array access".to_string());
        }
        self.advance();

        if !self.check(Token::Assign) {
            return Err("Expected = for array assignment".to_string());
        }
        self.advance();

        let value = Box::new(self.parse_expression()?);

        self.skip_newlines();
        Ok(Statement::ArrayAssign {
            name,
            index: Box::new(index),
            value,
        })
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.advance(); // Consume IF

        let condition = self.parse_expression()?;

        self.skip_newlines();
        if self.check(Token::Then) {
            self.advance();
        }
        self.skip_newlines();

        let branch_stop = [Token::Else, Token::ElseIf, Token::EndIf];
        let then_branch = self.parse_block_statements(&branch_stop);

        let mut else_if_branches = Vec::new();
        while self.check(Token::ElseIf) {
            self.advance(); // Consume ELSEIF

            let elseif_condition = self.parse_expression()?;

            self.skip_newlines();
            if self.check(Token::Then) {
                self.advance();
            }
            self.skip_newlines();

            let elseif_body = self.parse_block_statements(&branch_stop);

            else_if_branches.push((elseif_condition, elseif_body));
        }

        let mut else_branch = Vec::new();
        if self.check(Token::Else) {
            self.advance(); // Consume ELSE
            self.skip_newlines();

            else_branch = self.parse_block_statements(&[Token::EndIf]);
        }

        if !self.check(Token::EndIf) {
            return Err("Expected ENDIF".to_string());
        }
        self.advance(); // Consume ENDIF
        self.skip_newlines();

        Ok(Statement::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        })
    }

    fn parse_for(&mut self) -> Result<Statement, String> {
        self.advance();
        self.skip_newlines();

        let variable = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err("Expected loop variable".to_string());
        };
        self.advance();
        self.skip_newlines();

        if !self.check(Token::Assign) {
            return Err("Expected = in FOR loop".to_string());
        }
        self.advance();
        self.skip_newlines();

        let start = self.parse_expression()?;
        self.skip_newlines();

        if !self.check(Token::To) {
            return Err("Expected TO in FOR loop".to_string());
        }
        self.advance();
        self.skip_newlines();

        let end = self.parse_expression()?;
        self.skip_newlines();

        let body = self.parse_block_statements(&[Token::EndFor]);

        if !self.check(Token::EndFor) {
            return Err("Expected ENDFOR".to_string());
        }
        self.advance();
        self.skip_newlines();

        Ok(Statement::ForLoop {
            variable,
            start,
            end,
            body,
        })
    }

    fn parse_while(&mut self) -> Result<Statement, String> {
        self.advance();
        self.skip_newlines();

        let condition = self.parse_expression()?;
        self.skip_newlines();

        let body = self.parse_block_statements(&[Token::EndWhile]);

        if !self.check(Token::EndWhile) {
            return Err("Expected ENDWHILE".to_string());
        }
        self.advance();
        self.skip_newlines();

        Ok(Statement::WhileLoop { condition, body })
    }

    fn parse_declare(&mut self) -> Result<Statement, String> {
        self.advance();
        self.skip_newlines();

        let name = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err("Expected array name after DECLARE".to_string());
        };
        self.advance();
        self.skip_newlines();

        if !self.check(Token::LeftBracket) {
            return Err("Expected [ for array size".to_string());
        }
        self.advance();

        let size = match self.current() {
            Token::Number(n) => n as usize,
            _ => return Err("Expected number for array size".to_string()),
        };
        self.advance();

        if !self.check(Token::RightBracket) {
            return Err("Expected ] for array size".to_string());
        }
        self.advance();

        self.skip_newlines();
        Ok(Statement::DeclareArray { name, size })
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and()?;

        while self.check(Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: Operator::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;

        while self.check(Token::And) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: Operator::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_addition()?;

        loop {
            let op = match self.current() {
                Token::Eq => Operator::Equal,
                Token::NotEqual => Operator::NotEqual,
                Token::Assign => Operator::Equal,
                Token::LessThan => Operator::LessThan,
                Token::GreaterThan => Operator::GreaterThan,
                Token::LessEqual => Operator::LessEqual,
                Token::GreaterEqual => Operator::GreaterEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplication()?;

        loop {
            let op = match self.current() {
                Token::Plus => Operator::Add,
                Token::Minus => Operator::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_modulo()?;

        loop {
            let op = match self.current() {
                Token::Star => Operator::Multiply,
                Token::Slash => Operator::Divide,
                _ => break,
            };
            self.advance();
            let right = self.parse_modulo()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_modulo(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_power()?;

        while self.check(Token::Percent) {
            self.advance();
            let right = self.parse_power()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: Operator::Modulo,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;

        while self.check(Token::Caret) {
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: Operator::Power,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.check(Token::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expression::UnaryOp {
                operator: UnaryOperator::Negate,
                expr: Box::new(expr),
            });
        }

        if self.check(Token::Not) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expression::UnaryOp {
                operator: UnaryOperator::Not,
                expr: Box::new(expr),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        self.skip_newlines();

        match self.current() {
            Token::Number(n) => {
                self.advance();
                Ok(Expression::Number(n))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(Expression::String(s.clone()))
            }
            Token::Boolean(b) => {
                self.advance();
                Ok(Expression::Boolean(b))
            }
            Token::Identifier(name) => {
                self.advance();
                if self.check(Token::LeftBracket) {
                    self.advance();
                    let index = self.parse_expression()?;
                    if !self.check(Token::RightBracket) {
                        return Err("Expected ]".to_string());
                    }
                    self.advance();
                    Ok(Expression::ArrayAccess {
                        name: name.clone(),
                        index: Box::new(index),
                    })
                } else {
                    Ok(Expression::Identifier(name.clone()))
                }
            }
            token => Err(format!("Unexpected token: {}", token.to_string())),
        }
    }

    fn current(&self) -> Token {
        if self.position < self.tokens.len() {
            self.tokens[self.position].clone()
        } else {
            Token::EOF
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn check(&self, token: Token) -> bool {
        self.current() == token
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || self.current() == Token::EOF
    }

    fn skip_newlines(&mut self) {
        while self.check(Token::Newline) {
            self.advance();
        }
    }

    fn peek_is_identifier(&self) -> bool {
        matches!(self.current(), Token::Identifier(_))
    }

    fn peek_next_is_assign(&self) -> bool {
        if self.position + 1 < self.tokens.len() {
            self.tokens[self.position + 1] == Token::Assign
        } else {
            false
        }
    }

    fn peek_is_array_access(&self) -> bool {
        if self.position + 1 < self.tokens.len() {
            if let Token::Identifier(_) = self.current() {
                return self.tokens[self.position + 1] == Token::LeftBracket;
            }
        }
        false
    }
}
