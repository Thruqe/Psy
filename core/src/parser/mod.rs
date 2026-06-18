pub mod ast;

use self::ast::{
    Expression, ModuleImport, Operator, OutputValue, Spanned, Statement, UnaryOperator,
};
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

    pub fn parse(&mut self) -> (Vec<Spanned<Statement>>, Vec<ParseError>) {
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

    fn parse_imports(&mut self) -> Vec<Spanned<Statement>> {
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

    fn parse_import(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
        self.advance(); // Consume IMPORT

        let mut modules = Vec::new();
        modules.push(self.parse_module_import()?);

        while self.check(Token::Comma) {
            self.advance();
            modules.push(self.parse_module_import()?);
        }

        self.skip_newlines();
        Ok(Spanned::new(Statement::Import { modules }, line, column))
    }

    fn parse_module_import(&mut self) -> Result<ModuleImport, ParseError> {
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

        Ok(ModuleImport { name, functions })
    }

    fn parse_block_statements(&mut self, stop_tokens: &[Token]) -> Vec<Spanned<Statement>> {
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

    fn parse_statement(&mut self) -> Result<Spanned<Statement>, ParseError> {
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
        } else if self.check(Token::Function) {
            self.parse_function_declaration()
        } else if self.check(Token::Return) {
            self.parse_return()
        } else if self.check(Token::Const) {
            self.parse_const_declaration()
        } else if self.check(Token::Static) {
            self.parse_static_declaration()
        } else if self.check(Token::Pub) {
            self.parse_public_declaration()
        } else if self.peek_is_identifier() && self.peek_next_is_assign() {
            self.parse_assign()
        } else if self.peek_is_array_access() {
            self.parse_array_assign()
        } else if self.peek_is_call() {
            self.parse_expression_statement()
        } else {
            let (line, column) = self.current_pos();
            let token = self.current();
            self.advance();
            Err(ParseError {
                message: format!("Unknown statement starting with: {}", token.to_string()),
                line,
                column,
                severity: Severity::Error,
            })
        }
    }

    fn parse_input(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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
        Ok(Spanned::new(Statement::Input { variables }, line, column))
    }

    fn parse_output(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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
        Ok(Spanned::new(Statement::Output { values }, line, column))
    }

    fn parse_assign(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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
        Ok(Spanned::new(
            Statement::Assign {
                variable: var_name,
                expression: expr,
            },
            line,
            column,
        ))
    }

    fn parse_array_assign(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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
        Ok(Spanned::new(
            Statement::ArrayAssign {
                name,
                index: Box::new(index),
                value,
            },
            line,
            column,
        ))
    }

    fn parse_if(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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

        Ok(Spanned::new(
            Statement::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            },
            line,
            column,
        ))
    }

    fn parse_for(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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

        Ok(Spanned::new(
            Statement::ForLoop {
                variable,
                start,
                end,
                body,
            },
            line,
            column,
        ))
    }

    fn parse_while(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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

        Ok(Spanned::new(
            Statement::WhileLoop { condition, body },
            line,
            column,
        ))
    }

    fn parse_declare(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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
        Ok(Spanned::new(
            Statement::DeclareArray { name, size },
            line,
            column,
        ))
    }

    fn parse_expression(&mut self) -> Result<Spanned<Expression>, ParseError> {
        self.parse_or()
    }

    fn parse_function_declaration(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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

        Ok(Spanned::new(
            Statement::FunctionDeclaration {
                name,
                parameters,
                body,
            },
            line,
            column,
        ))
    }

    fn parse_return(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
        self.advance(); // Consume RETURN

        if self.is_at_end() || self.check(Token::Newline) {
            self.skip_newlines();
            return Ok(Spanned::new(
                Statement::Return { value: None },
                line,
                column,
            ));
        }

        let expr = self.parse_expression()?;
        self.skip_newlines();
        Ok(Spanned::new(
            Statement::Return { value: Some(expr) },
            line,
            column,
        ))
    }

    fn parse_const_declaration(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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

        Ok(Spanned::new(
            Statement::ConstDeclaration {
                name,
                expression: expr,
            },
            line,
            column,
        ))
    }

    fn parse_static_declaration(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
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

        Ok(Spanned::new(
            Statement::StaticDeclaration {
                name,
                expression: expr,
            },
            line,
            column,
        ))
    }

    fn parse_public_declaration(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
        self.advance(); // Consume PUB

        let inner = if self.check(Token::Function) {
            self.parse_function_declaration()?
        } else if self.check(Token::Declare) {
            self.parse_declare()?
        } else if self.check(Token::Const) {
            self.parse_const_declaration()?
        } else {
            return Err(
                self.error_at_current("PUB can only be used with FUNCTION, DECLARE, or CONST")
            );
        };

        Ok(Spanned::new(
            Statement::Public(Box::new(inner)),
            line,
            column,
        ))
    }

    fn parse_expression_statement(&mut self) -> Result<Spanned<Statement>, ParseError> {
        let (line, column) = self.current_pos();
        let expr = self.parse_expression()?;
        self.skip_newlines();
        Ok(Spanned::new(
            Statement::ExpressionStatement(expr),
            line,
            column,
        ))
    }

    fn parse_or(&mut self) -> Result<Spanned<Expression>, ParseError> {
        let (line, column) = self.current_pos();
        let mut left = self.parse_and()?;

        while self.check(Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Spanned::new(
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::Or,
                    right: Box::new(right),
                },
                line,
                column,
            );
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Spanned<Expression>, ParseError> {
        let (line, column) = self.current_pos();
        let mut left = self.parse_comparison()?;

        while self.check(Token::And) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Spanned::new(
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::And,
                    right: Box::new(right),
                },
                line,
                column,
            );
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Spanned<Expression>, ParseError> {
        let (line, column) = self.current_pos();
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
            left = Spanned::new(
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                },
                line,
                column,
            );
        }

        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Spanned<Expression>, ParseError> {
        let (line, column) = self.current_pos();
        let mut left = self.parse_multiplication()?;

        loop {
            let op = match self.current() {
                Token::Plus => Operator::Add,
                Token::Minus => Operator::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Spanned::new(
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                },
                line,
                column,
            );
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Spanned<Expression>, ParseError> {
        let (line, column) = self.current_pos();
        let mut left = self.parse_modulo()?;

        loop {
            let op = match self.current() {
                Token::Star => Operator::Multiply,
                Token::Slash => Operator::Divide,
                _ => break,
            };
            self.advance();
            let right = self.parse_modulo()?;
            left = Spanned::new(
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                },
                line,
                column,
            );
        }

        Ok(left)
    }

    fn parse_modulo(&mut self) -> Result<Spanned<Expression>, ParseError> {
        let (line, column) = self.current_pos();
        let mut left = self.parse_power()?;

        while self.check(Token::Percent) {
            self.advance();
            let right = self.parse_power()?;
            left = Spanned::new(
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::Modulo,
                    right: Box::new(right),
                },
                line,
                column,
            );
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Spanned<Expression>, ParseError> {
        let (line, column) = self.current_pos();
        let mut left = self.parse_unary()?;

        while self.check(Token::Caret) {
            self.advance();
            let right = self.parse_unary()?;
            left = Spanned::new(
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::Power,
                    right: Box::new(right),
                },
                line,
                column,
            );
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Spanned<Expression>, ParseError> {
        let (line, column) = self.current_pos();

        if self.check(Token::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Spanned::new(
                Expression::UnaryOp {
                    operator: UnaryOperator::Negate,
                    expr: Box::new(expr),
                },
                line,
                column,
            ));
        }

        if self.check(Token::Not) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Spanned::new(
                Expression::UnaryOp {
                    operator: UnaryOperator::Not,
                    expr: Box::new(expr),
                },
                line,
                column,
            ));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Spanned<Expression>, ParseError> {
        self.skip_newlines();
        let (line, column) = self.current_pos();

        match self.current() {
            Token::Number(n) => {
                self.advance();
                Ok(Spanned::new(Expression::Number(n), line, column))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(Spanned::new(Expression::String(s.clone()), line, column))
            }
            Token::Boolean(b) => {
                self.advance();
                Ok(Spanned::new(Expression::Boolean(b), line, column))
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
                Ok(Spanned::new(
                    Expression::ArrayLiteral(elements),
                    line,
                    column,
                ))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                if !self.check(Token::RightParen) {
                    return Err(self.error_at_current("Expected )"));
                }
                self.advance();
                Ok(expr)
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
                    Ok(Spanned::new(
                        Expression::ArrayAccess {
                            name: name.clone(),
                            index: Box::new(index),
                        },
                        line,
                        column,
                    ))
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
                    Ok(Spanned::new(
                        Expression::FunctionCall {
                            name: name.clone(),
                            arguments,
                        },
                        line,
                        column,
                    ))
                } else {
                    Ok(Spanned::new(
                        Expression::Identifier(name.clone()),
                        line,
                        column,
                    ))
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
