use crate::interpreter::environment::{Environment, Value};
use crate::parser::ast::{Expression, Operator, OutputValue, Statement, UnaryOperator};
use std::io::{self, Write};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn run(&mut self, statements: &[Statement]) -> Result<(), String> {
        for stmt in statements {
            self.execute_statement(stmt)?;
        }
        Ok(())
    }

    pub fn print_state(&self) {
        self.environment.print_state();
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Assign {
                variable,
                expression,
            } => {
                let value = self.evaluate_expression(expression)?;
                self.environment.set(variable, value);
            }
            Statement::Input { variables } => {
                for var in variables {
                    print!("Enter {}: ", var);
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim();

                    // Try to parse as number first
                    if let Ok(num) = input.parse::<f64>() {
                        self.environment.set(var, Value::Number(num));
                    } else {
                        self.environment.set(var, Value::String(input.to_string()));
                    }
                }
            }
            Statement::Output { values } => {
                for value in values {
                    match value {
                        OutputValue::Expression(expr) => {
                            let val = self.evaluate_expression(expr)?;
                            print!("{}", self.value_to_string(&val));
                        }
                        OutputValue::StringLiteral(s) => {
                            print!("{}", s);
                        }
                    }
                }
                println!();
            }
            Statement::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                let cond = self.evaluate_expression(condition)?;
                let is_true = self.is_truthy(&cond);

                if is_true {
                    for stmt in then_branch {
                        self.execute_statement(stmt)?;
                    }
                } else {
                    let mut executed = false;
                    for (elseif_cond, elseif_body) in else_if_branches {
                        let elseif_cond_value = self.evaluate_expression(elseif_cond)?;
                        if self.is_truthy(&elseif_cond_value) {
                            for stmt in elseif_body {
                                self.execute_statement(stmt)?;
                            }
                            executed = true;
                            break;
                        }
                    }
                    if !executed {
                        for stmt in else_branch {
                            self.execute_statement(stmt)?;
                        }
                    }
                }
            }
            Statement::ForLoop {
                variable,
                start,
                end,
                body,
            } => {
                let start_val = self.evaluate_expression(start)?;
                let end_val = self.evaluate_expression(end)?;

                if let (Value::Number(s), Value::Number(e)) = (start_val, end_val) {
                    let start_int = s as i32;
                    let end_int = e as i32;

                    for i in start_int..=end_int {
                        self.environment.set(variable, Value::Number(i as f64));
                        for stmt in body {
                            self.execute_statement(stmt)?;
                        }
                    }
                } else {
                    return Err("FOR loop bounds must be numbers".to_string());
                }
            }
            Statement::WhileLoop { condition, body } => {
                let mut cond = self.evaluate_expression(condition)?;
                let mut is_true = self.is_truthy(&cond);

                while is_true {
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                    cond = self.evaluate_expression(condition)?;
                    is_true = self.is_truthy(&cond);
                }
            }
            Statement::DeclareArray { name, size } => {
                self.environment.declare_array(name, *size);
            }
            Statement::ArrayAssign { name, index, value } => {
                let idx_val = self.evaluate_expression(index)?;
                let val = self.evaluate_expression(value)?;

                if let Value::Number(idx) = idx_val {
                    self.environment.set_array_element(name, idx as usize, val);
                } else {
                    return Err("Array index must be a number".to_string());
                }
            }
        }
        Ok(())
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Number(n) => Ok(Value::Number(*n)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Boolean(b) => Ok(Value::Boolean(*b)),
            Expression::Identifier(name) => Ok(self.environment.get(name)),
            Expression::ArrayAccess { name, index } => {
                let idx_val = self.evaluate_expression(index)?;
                if let Value::Number(idx) = idx_val {
                    Ok(self.environment.get_array_element(name, idx as usize))
                } else {
                    Err("Array index must be a number".to_string())
                }
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;

                match operator {
                    Operator::Add => match (left_val, right_val) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                        (Value::String(l), Value::Number(r)) => {
                            Ok(Value::String(l + &r.to_string()))
                        }
                        (Value::Number(l), Value::String(r)) => {
                            Ok(Value::String(l.to_string() + &r))
                        }
                        _ => Err("Invalid addition".to_string()),
                    },
                    Operator::Subtract => {
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Number(l - r))
                        } else {
                            Err("Subtraction requires numbers".to_string())
                        }
                    }
                    Operator::Multiply => {
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Number(l * r))
                        } else {
                            Err("Multiplication requires numbers".to_string())
                        }
                    }
                    Operator::Divide => {
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            if r != 0.0 {
                                Ok(Value::Number(l / r))
                            } else {
                                Err("Division by zero".to_string())
                            }
                        } else {
                            Err("Division requires numbers".to_string())
                        }
                    }
                    Operator::Modulo => {
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            if r != 0.0 {
                                Ok(Value::Number(l % r))
                            } else {
                                Err("Modulo by zero".to_string())
                            }
                        } else {
                            Err("Modulo requires numbers".to_string())
                        }
                    }
                    Operator::Power => {
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Number(l.powf(r)))
                        } else {
                            Err("Power requires numbers".to_string())
                        }
                    }
                    Operator::Equal => Ok(Value::Boolean(left_val == right_val)),
                    Operator::NotEqual => Ok(Value::Boolean(left_val != right_val)),
                    Operator::LessThan => {
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Boolean(l < r))
                        } else {
                            Err("Comparison requires numbers".to_string())
                        }
                    }
                    Operator::GreaterThan => {
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Boolean(l > r))
                        } else {
                            Err("Comparison requires numbers".to_string())
                        }
                    }
                    Operator::LessEqual => {
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Boolean(l <= r))
                        } else {
                            Err("Comparison requires numbers".to_string())
                        }
                    }
                    Operator::GreaterEqual => {
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Boolean(l >= r))
                        } else {
                            Err("Comparison requires numbers".to_string())
                        }
                    }
                    Operator::And => {
                        let l_bool = self.is_truthy(&left_val);
                        let r_bool = self.is_truthy(&right_val);
                        Ok(Value::Boolean(l_bool && r_bool))
                    }
                    Operator::Or => {
                        let l_bool = self.is_truthy(&left_val);
                        let r_bool = self.is_truthy(&right_val);
                        Ok(Value::Boolean(l_bool || r_bool))
                    }
                }
            }
            Expression::UnaryOp { operator, expr } => {
                let val = self.evaluate_expression(expr)?;
                match operator {
                    UnaryOperator::Negate => {
                        if let Value::Number(n) = val {
                            Ok(Value::Number(-n))
                        } else {
                            Err("Negation requires a number".to_string())
                        }
                    }
                    UnaryOperator::Not => Ok(Value::Boolean(!self.is_truthy(&val))),
                }
            }
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Undefined => false,
        }
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => format!("{:?}", arr),
            Value::Undefined => "undefined".to_string(),
        }
    }
}
