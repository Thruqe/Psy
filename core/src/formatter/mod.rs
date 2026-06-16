use crate::parser::ast::{Expression, Operator, OutputValue, Statement};
use std::fmt::Write;

pub struct Formatter {
    indent_level: usize,
    indent_size: usize,
    output: String,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            indent_level: 0,
            indent_size: 4,
            output: String::new(),
        }
    }

    pub fn format(&mut self, source: &str) -> Result<String, String> {
        // First, parse the source
        use crate::lexer::Lexer;
        use crate::parser::Parser;

        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _errors) = parser.parse(); // Destructure the tuple

        // Format the AST
        for stmt in &ast {
            self.format_statement(stmt)?;
        }

        Ok(self.output.clone())
    }

    fn format_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        let indent = " ".repeat(self.indent_level * self.indent_size);

        match stmt {
            Statement::Assign {
                variable,
                expression,
            } => {
                let expr_str = self.format_expression(expression)?;
                writeln!(self.output, "{}{} = {}", indent, variable, expr_str)
                    .map_err(|e| e.to_string())?;
            }
            Statement::Input { variables } => {
                let vars = variables.join(", ");
                writeln!(self.output, "{}INPUT {}", indent, vars).map_err(|e| e.to_string())?;
            }
            Statement::Output { values } => {
                let mut output_parts = Vec::new();
                for value in values {
                    match value {
                        OutputValue::Expression(expr) => {
                            output_parts.push(self.format_expression(expr)?);
                        }
                        OutputValue::StringLiteral(s) => {
                            output_parts.push(format!("\"{}\"", s));
                        }
                    }
                }
                let output_line = output_parts.join(" ");
                writeln!(self.output, "{}OUTPUT {}", indent, output_line)
                    .map_err(|e| e.to_string())?;
            }
            Statement::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                // IF condition THEN
                let cond_str = self.format_expression(condition)?;
                writeln!(self.output, "{}IF {} THEN", indent, cond_str)
                    .map_err(|e| e.to_string())?;

                // Then branch
                self.indent_level += 1;
                for stmt in then_branch {
                    self.format_statement(stmt)?;
                }
                self.indent_level -= 1;

                // ELSEIF branches
                for (elseif_cond, elseif_body) in else_if_branches {
                    let elseif_str = self.format_expression(elseif_cond)?;
                    writeln!(self.output, "{}ELSEIF {} THEN", indent, elseif_str)
                        .map_err(|e| e.to_string())?;

                    self.indent_level += 1;
                    for stmt in elseif_body {
                        self.format_statement(stmt)?;
                    }
                    self.indent_level -= 1;
                }

                // ELSE branch
                if !else_branch.is_empty() {
                    writeln!(self.output, "{}ELSE", indent).map_err(|e| e.to_string())?;

                    self.indent_level += 1;
                    for stmt in else_branch {
                        self.format_statement(stmt)?;
                    }
                    self.indent_level -= 1;
                }

                writeln!(self.output, "{}ENDIF", indent).map_err(|e| e.to_string())?;
            }
            Statement::ForLoop {
                variable,
                start,
                end,
                body,
            } => {
                let start_str = self.format_expression(start)?;
                let end_str = self.format_expression(end)?;
                writeln!(
                    self.output,
                    "{}FOR {} = {} TO {}",
                    indent, variable, start_str, end_str
                )
                .map_err(|e| e.to_string())?;

                self.indent_level += 1;
                for stmt in body {
                    self.format_statement(stmt)?;
                }
                self.indent_level -= 1;

                writeln!(self.output, "{}ENDFOR", indent).map_err(|e| e.to_string())?;
            }
            Statement::WhileLoop { condition, body } => {
                let cond_str = self.format_expression(condition)?;
                writeln!(self.output, "{}WHILE {}", indent, cond_str).map_err(|e| e.to_string())?;

                self.indent_level += 1;
                for stmt in body {
                    self.format_statement(stmt)?;
                }
                self.indent_level -= 1;

                writeln!(self.output, "{}ENDWHILE", indent).map_err(|e| e.to_string())?;
            }
            Statement::DeclareArray { name, size } => {
                writeln!(self.output, "{}DECLARE {}[{}]", indent, name, size)
                    .map_err(|e| e.to_string())?;
            }
            Statement::ArrayAssign { name, index, value } => {
                let index_str = self.format_expression(index)?;
                let value_str = self.format_expression(value)?;
                writeln!(
                    self.output,
                    "{}{}[{}] = {}",
                    indent, name, index_str, value_str
                )
                .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    fn format_expression(&self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Number(n) => Ok(n.to_string()),
            Expression::String(s) => Ok(format!("\"{}\"", s)),
            Expression::Boolean(b) => Ok(b.to_string().to_uppercase()),
            Expression::Identifier(name) => Ok(name.clone()),
            Expression::ArrayAccess { name, index } => {
                let idx = self.format_expression(index)?;
                Ok(format!("{}[{}]", name, idx))
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left_str = self.format_expression(left)?;
                let right_str = self.format_expression(right)?;
                let op_str = match operator {
                    Operator::Add => " + ",
                    Operator::Subtract => " - ",
                    Operator::Multiply => "*",
                    Operator::Divide => "/",
                    Operator::Modulo => " % ",
                    Operator::Power => "^",
                    Operator::Equal => " == ",
                    Operator::NotEqual => " != ",
                    Operator::LessThan => " < ",
                    Operator::GreaterThan => " > ",
                    Operator::LessEqual => " <= ",
                    Operator::GreaterEqual => " >= ",
                    Operator::And => " AND ",
                    Operator::Or => " OR ",
                };
                Ok(format!("{}{}{}", left_str, op_str, right_str))
            }
            Expression::UnaryOp { operator, expr } => {
                let expr_str = self.format_expression(expr)?;
                match operator {
                    crate::parser::ast::UnaryOperator::Negate => Ok(format!("-{}", expr_str)),
                    crate::parser::ast::UnaryOperator::Not => Ok(format!("NOT {}", expr_str)),
                }
            }
        }
    }
}
