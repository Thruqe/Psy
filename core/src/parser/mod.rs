pub mod ast;

use self::ast::{Expression, Operator, OutputValue, Statement, UnaryOperator};
use crate::lexer::{PositionedToken, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (line {}, column {})",
            self.message, self.line, self.column
        )
    }
}

pub struct Parser {
    tokens: Vec<PositionedToken>,
    position: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<PositionedToken>) -> Self {
        Parser {
            tokens,
            position: 0,
            errors: Vec::new(),
        }
    }

    /// Parses the program, collecting any errors encountered along the way
    /// instead of printing them. Returns the parsed statements plus every
    /// error/warning recorded during parsing.
    pub fn parse(&mut self) -> (Vec<Statement>, Vec<ParseError>) {
        self.skip_newlines();

        let imports = self.parse_imports();

        if self.check(Token::Start) {
            self.advance();
            self.skip_newlines();
        } else {
            self.push_warning_at_current("Program should start with START");
        }

        let mut statements = imports;
        statements.extend(self.parse_block_statements(&[Token::End]));

        if self.check(Token::End) {
            self.advance();
        } else {
            self.push_warning_at_current("Program should end with END");
        }

        (statements, std::mem::take(&mut self.errors))
    }

    /// Consumes zero or more IMPORT statements appearing before START.
    /// Each IMPORT line becomes one Statement::Import.
    fn parse_imports(&mut self) -> Vec<Statement> {
        let mut imports = Vec::new();

        while self.check(Token::Import) {
            match self.parse_import() {
                Ok(stmt) => imports.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.recover_to_boundary(&[Token::Start]);
                }
            }
            self.skip_newlines();
        }

        imports
    }

    fn parse_import(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Consume IMPORT

        let mut modules = Vec::new();
        modules.push(self.parse_module_import()?);

        while self.check(Token::Comma) {
            self.advance();
            modules.push(self.parse_module_import()?);
        }

        self.skip_newlines();
        Ok(Statement::Import { modules })
    }

    fn parse_module_import(&mut self) -> Result<ast::ModuleImport, ParseError> {
        let name = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err(self.error_at_current("Expected module name after IMPORT"));
        };
        self.advance();

        let functions = if self.check(Token::LeftBracket) {
            self.advance();
            let mut names = Vec::new();

            while !self.check(Token::RightBracket) {
                if let Token::Identifier(func_name) = self.current() {
                    names.push(func_name.clone());
                    self.advance();
                    if self.check(Token::Comma) {
                        self.advance();
                    }
                } else {
                    return Err(self.error_at_current("Expected function name in import list"));
                }
            }
            self.advance(); // Consume ]
            Some(names)
        } else {
            None
        };

        Ok(ast::ModuleImport { name, functions })
    }

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
                    self.errors.push(e);
                    self.recover_to_boundary(stop_tokens);
                }
            }
        }

        statements
    }

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

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        self.skip_newlines();

        if self.is_at_end() {
            return Err(self.error_at_current("Unexpected end of file"));
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
        } else if self.check(Token::Function) {
            self.parse_function_declaration()
        } else if self.check(Token::Return) {
            self.parse_return()
        } else if self.check(Token::Const) {
            self.parse_const_declaration()
        } else if self.check(Token::Static) {
            self.parse_static_declaration()
        } else if self.peek_is_call() {
            self.parse_expression_statement()
        } else {
            let token = self.current();
            let err = self.error_at_current(&format!(
                "Unknown statement starting with: {}",
                token.to_string()
            ));
            self.advance();
            Err(err)
        }
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expression()?;
        self.skip_newlines();
        Ok(Statement::ExpressionStatement(expr))
    }

    fn parse_input(&mut self) -> Result<Statement, ParseError> {
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
                return Err(self.error_at_current("Expected identifier after INPUT"));
            }
        }

        self.skip_newlines();
        Ok(Statement::Input { variables })
    }

    fn parse_output(&mut self) -> Result<Statement, ParseError> {
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

    fn parse_assign(&mut self) -> Result<Statement, ParseError> {
        let var_name = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err(self.error_at_current("Expected identifier"));
        };

        self.advance();
        self.skip_newlines();

        if !self.check(Token::Assign) {
            return Err(self.error_at_current("Expected = for assignment"));
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

    fn parse_array_assign(&mut self) -> Result<Statement, ParseError> {
        let name = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err(self.error_at_current("Expected array name"));
        };
        self.advance();

        if !self.check(Token::LeftBracket) {
            return Err(self.error_at_current("Expected [ for array access"));
        }
        self.advance();

        let index = self.parse_expression()?;

        if !self.check(Token::RightBracket) {
            return Err(self.error_at_current("Expected ] for array access"));
        }
        self.advance();

        if !self.check(Token::Assign) {
            return Err(self.error_at_current("Expected = for array assignment"));
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

    fn parse_const_declaration(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Consume CONST

        let name = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err(self.error_at_current("Expected identifier after CONST"));
        };
        self.advance();

        if !self.check(Token::Assign) {
            return Err(self.error_at_current("Expected = after CONST name"));
        }
        self.advance();

        let expr = self.parse_expression()?;
        self.skip_newlines();

        Ok(Statement::ConstDeclaration {
            name,
            expression: expr,
        })
    }

    fn parse_static_declaration(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Consume STATIC

        let name = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err(self.error_at_current("Expected identifier after STATIC"));
        };
        self.advance();

        if !self.check(Token::Assign) {
            return Err(self.error_at_current("Expected = after STATIC name"));
        }
        self.advance();

        let expr = self.parse_expression()?;
        self.skip_newlines();

        Ok(Statement::StaticDeclaration {
            name,
            expression: expr,
        })
    }

    fn parse_if(&mut self) -> Result<Statement, ParseError> {
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
            return Err(self.error_at_current("Expected ENDIF"));
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

    fn parse_for(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        self.skip_newlines();

        let variable = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err(self.error_at_current("Expected loop variable"));
        };
        self.advance();
        self.skip_newlines();

        if !self.check(Token::Assign) {
            return Err(self.error_at_current("Expected = in FOR loop"));
        }
        self.advance();
        self.skip_newlines();

        let start = self.parse_expression()?;
        self.skip_newlines();

        if !self.check(Token::To) {
            return Err(self.error_at_current("Expected TO in FOR loop"));
        }
        self.advance();
        self.skip_newlines();

        let end = self.parse_expression()?;
        self.skip_newlines();

        let body = self.parse_block_statements(&[Token::EndFor]);

        if !self.check(Token::EndFor) {
            return Err(self.error_at_current("Expected ENDFOR"));
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

    fn parse_while(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        self.skip_newlines();

        let condition = self.parse_expression()?;
        self.skip_newlines();

        let body = self.parse_block_statements(&[Token::EndWhile]);

        if !self.check(Token::EndWhile) {
            return Err(self.error_at_current("Expected ENDWHILE"));
        }
        self.advance();
        self.skip_newlines();

        Ok(Statement::WhileLoop { condition, body })
    }

    fn parse_declare(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        self.skip_newlines();

        let name = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err(self.error_at_current("Expected array name after DECLARE"));
        };
        self.advance();
        self.skip_newlines();

        if !self.check(Token::LeftBracket) {
            return Err(self.error_at_current("Expected [ for array size"));
        }
        self.advance();

        let size = match self.current() {
            Token::Number(n) => n as usize,
            _ => return Err(self.error_at_current("Expected number for array size")),
        };
        self.advance();

        if !self.check(Token::RightBracket) {
            return Err(self.error_at_current("Expected ] for array size"));
        }
        self.advance();

        self.skip_newlines();
        Ok(Statement::DeclareArray { name, size })
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_or()
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Consume FUNCTION

        let name = if let Token::Identifier(name) = self.current() {
            name.clone()
        } else {
            return Err(self.error_at_current("Expected function name after FUNCTION"));
        };
        self.advance();

        if !self.check(Token::LeftParen) {
            return Err(self.error_at_current("Expected ( after function name"));
        }
        self.advance();

        let mut parameters = Vec::new();
        if !self.check(Token::RightParen) {
            loop {
                if let Token::Identifier(param) = self.current() {
                    parameters.push(param.clone());
                    self.advance();
                } else {
                    return Err(self.error_at_current("Expected parameter name"));
                }

                if self.check(Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if !self.check(Token::RightParen) {
            return Err(self.error_at_current("Expected , or ) in parameter list"));
        }
        self.advance(); // Consume )
        self.skip_newlines();

        let body = self.parse_block_statements(&[Token::EndFunction]);

        if !self.check(Token::EndFunction) {
            return Err(self.error_at_current("Expected ENDFUNCTION"));
        }
        self.advance();
        self.skip_newlines();

        Ok(Statement::FunctionDeclaration {
            name,
            parameters,
            body,
        })
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // Consume RETURN

        if self.is_at_end() || self.check(Token::Newline) {
            self.skip_newlines();
            return Ok(Statement::Return { value: None });
        }

        let expr = self.parse_expression()?;
        self.skip_newlines();
        Ok(Statement::Return { value: Some(expr) })
    }

    fn parse_or(&mut self) -> Result<Expression, ParseError> {
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

    fn parse_and(&mut self) -> Result<Expression, ParseError> {
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

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
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

    fn parse_addition(&mut self) -> Result<Expression, ParseError> {
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

    fn parse_multiplication(&mut self) -> Result<Expression, ParseError> {
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

    fn parse_modulo(&mut self) -> Result<Expression, ParseError> {
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

    fn parse_power(&mut self) -> Result<Expression, ParseError> {
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

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
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

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
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
            Token::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();

                while !self.check(Token::RightBracket) {
                    elements.push(self.parse_expression()?);
                    if self.check(Token::Comma) {
                        self.advance();
                    }
                }
                self.advance(); // Consume ]
                Ok(Expression::ArrayLiteral(elements))
            }
            Token::Identifier(name) => {
                self.advance();
                if self.check(Token::LeftBracket) {
                    self.advance();
                    let index = self.parse_expression()?;
                    if !self.check(Token::RightBracket) {
                        return Err(self.error_at_current("Expected ]"));
                    }
                    self.advance();
                    Ok(Expression::ArrayAccess {
                        name: name.clone(),
                        index: Box::new(index),
                    })
                } else if self.check(Token::LeftParen) {
                    self.advance();
                    let mut arguments = Vec::new();
                    while !self.check(Token::RightParen) {
                        arguments.push(self.parse_expression()?);
                        if self.check(Token::Comma) {
                            self.advance();
                        }
                    }
                    self.advance(); // Consume )
                    Ok(Expression::FunctionCall {
                        name: name.clone(),
                        arguments,
                    })
                } else {
                    Ok(Expression::Identifier(name.clone()))
                }
            }
            token => {
                Err(self.error_at_current(&format!("Unexpected token: {}", token.to_string())))
            }
        }
    }

    fn current(&self) -> Token {
        if self.position < self.tokens.len() {
            self.tokens[self.position].token.clone()
        } else {
            Token::EOF
        }
    }

    fn current_pos(&self) -> (usize, usize) {
        if self.position < self.tokens.len() {
            let t = &self.tokens[self.position];
            (t.line, t.column)
        } else {
            self.tokens
                .last()
                .map(|t| (t.line, t.column))
                .unwrap_or((1, 1))
        }
    }

    fn error_at_current(&self, message: &str) -> ParseError {
        let (line, column) = self.current_pos();
        ParseError {
            message: message.to_string(),
            line,
            column,
            severity: Severity::Error,
        }
    }

    fn warning_at_current(&self, message: &str) -> ParseError {
        let (line, column) = self.current_pos();
        ParseError {
            message: message.to_string(),
            line,
            column,
            severity: Severity::Warning,
        }
    }

    fn push_warning_at_current(&mut self, message: &str) {
        let warn = self.warning_at_current(message);
        self.errors.push(warn);
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
            self.tokens[self.position + 1].token == Token::Assign
        } else {
            false
        }
    }

    fn peek_is_array_access(&self) -> bool {
        if self.position + 1 < self.tokens.len() {
            if let Token::Identifier(_) = self.current() {
                return self.tokens[self.position + 1].token == Token::LeftBracket;
            }
        }
        false
    }
    fn peek_is_call(&self) -> bool {
        if self.position + 1 < self.tokens.len() {
            if let Token::Identifier(_) = self.current() {
                return self.tokens[self.position + 1].token == Token::LeftParen;
            }
        }
        false
    }
}
