use crate::interpreter::environment::{Environment, Value};
use crate::interpreter::native::{self, get_module};
use crate::parser::ast::{Expression, Operator, OutputValue, Statement, UnaryOperator};
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Debug, Clone)]
struct FunctionDef {
    parameters: Vec<String>,
    body: Vec<Statement>,
}

/// Signals whether a block finished normally or hit a RETURN that needs
/// to propagate up through any number of enclosing IF/FOR/WHILE blocks
/// to the function call site.
enum ControlFlow {
    Normal,
    Return(Value),
}

pub struct Interpreter {
    environment: Environment,
    functions: HashMap<String, FunctionDef>,
    native_functions: HashMap<String, native::NativeFunctionInfo>,
    native_constants: HashMap<String, Value>,
    imported_modules: Vec<String>,
    statics: HashMap<(String, String), Value>,
    static_scope_stack: Vec<(String, Vec<String>)>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
            functions: HashMap::new(),
            native_functions: HashMap::new(),
            native_constants: HashMap::new(),
            imported_modules: Vec::new(),
            statics: HashMap::new(),
            static_scope_stack: Vec::new(),
        }
    }

    pub fn run(&mut self, statements: &[Statement]) -> Result<(), String> {
        // Process imports first
        for stmt in statements {
            if let Statement::Import { modules } = stmt {
                for module_import in modules {
                    if let Some(module) = get_module(&module_import.name) {
                        self.imported_modules.push(module_import.name.clone());

                        let mut funcs = module.functions;
                        if let Some(imported_funcs) = &module_import.functions {
                            funcs.retain(|k, _| imported_funcs.iter().any(|f| f == k));
                        }
                        for (name, info) in funcs {
                            self.native_functions.insert(name.to_string(), info);
                        }

                        let mut consts = module.constants;
                        if let Some(imported_funcs) = &module_import.functions {
                            consts.retain(|k, _| imported_funcs.iter().any(|f| f == k));
                        }
                        for (name, value) in consts {
                            self.environment.set_const(&name, value)?;
                        }
                    } else {
                        return Err(format!("Unknown module: {}", module_import.name));
                    }
                }
            }
        }
        // Pre-pass: register every function declaration before executing
        // anything, so forward references and recursion both work
        // regardless of declaration order.
        for stmt in statements {
            if let Statement::FunctionDeclaration {
                name,
                parameters,
                body,
            } = stmt
            {
                self.functions.insert(
                    name.clone(),
                    FunctionDef {
                        parameters: parameters.clone(),
                        body: body.clone(),
                    },
                );
            }
        }

        for stmt in statements {
            self.execute_statement(stmt)?;
        }
        Ok(())
    }

    pub fn print_state(&self) {
        self.environment.print_state();
    }

    /// Returns the persistent-storage key for `name` if it refers to a
    /// STATIC variable in the function currently executing (the top of
    /// the static scope stack), or None if it's an ordinary variable.
    fn current_static_key(&self, name: &str) -> Option<(String, String)> {
        let (func_name, static_names) = self.static_scope_stack.last()?;
        if static_names.iter().any(|n| n == name) {
            Some((func_name.clone(), name.to_string()))
        } else {
            None
        }
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<ControlFlow, String> {
        match stmt {
            Statement::Import { .. } => {
                // Already processed in the run() pre-pass
                Ok(ControlFlow::Normal)
            }
            Statement::Assign {
                variable,
                expression,
            } => {
                let value = self.evaluate_expression(expression)?;
                if let Some(key) = self.current_static_key(variable) {
                    self.statics.insert(key, value);
                } else {
                    self.environment.set(variable, value)?;
                }
                Ok(ControlFlow::Normal)
            }
            Statement::Input { variables } => {
                for var in variables {
                    print!("Enter {}: ", var);
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim();

                    if let Ok(num) = input.parse::<f64>() {
                        self.environment.set(var, Value::Number(num))?;
                    } else {
                        self.environment
                            .set(var, Value::String(input.to_string()))?;
                    }
                }
                Ok(ControlFlow::Normal)
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
                Ok(ControlFlow::Normal)
            }
            Statement::StaticDeclaration { .. } => {
                // All real work (init-once-ever, plus every read/write of
                // this name for the rest of the call) is handled by the
                // static scope stack set up in call_function, and by the
                // redirection in Assign/Identifier above. This statement
                // is purely positional in the AST by the time we get here.
                Ok(ControlFlow::Normal)
            }
            Statement::ExpressionStatement(expr) => {
                self.evaluate_expression(expr)?;
                Ok(ControlFlow::Normal)
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
                    self.execute_block(then_branch)
                } else {
                    let mut result = None;
                    for (elseif_cond, elseif_body) in else_if_branches {
                        let elseif_cond_value = self.evaluate_expression(elseif_cond)?;
                        if self.is_truthy(&elseif_cond_value) {
                            result = Some(self.execute_block(elseif_body)?);
                            break;
                        }
                    }
                    match result {
                        Some(cf) => Ok(cf),
                        None => self.execute_block(else_branch),
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
                        self.environment.set(variable, Value::Number(i as f64))?;
                        match self.execute_block(body)? {
                            ControlFlow::Normal => {}
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        }
                    }
                    Ok(ControlFlow::Normal)
                } else {
                    Err("FOR loop bounds must be numbers".to_string())
                }
            }
            Statement::WhileLoop { condition, body } => {
                let mut cond = self.evaluate_expression(condition)?;
                let mut is_true = self.is_truthy(&cond);

                while is_true {
                    match self.execute_block(body)? {
                        ControlFlow::Normal => {}
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                    }
                    cond = self.evaluate_expression(condition)?;
                    is_true = self.is_truthy(&cond);
                }
                Ok(ControlFlow::Normal)
            }
            Statement::ConstDeclaration { name, expression } => {
                let value = self.evaluate_expression(expression)?;
                self.environment.set_const(name, value)?;
                Ok(ControlFlow::Normal)
            }
            Statement::DeclareArray { name, size } => {
                self.environment.declare_array(name, *size);
                Ok(ControlFlow::Normal)
            }
            Statement::ArrayAssign { name, index, value } => {
                let idx_val = self.evaluate_expression(index)?;
                let val = self.evaluate_expression(value)?;

                if let Value::Number(idx) = idx_val {
                    self.environment.set_array_element(name, idx as usize, val);
                    Ok(ControlFlow::Normal)
                } else {
                    Err("Array index must be a number".to_string())
                }
            }
            Statement::FunctionDeclaration { .. } => {
                // Already registered in the pre-pass; nothing to do at
                // execution time when encountered inline.
                Ok(ControlFlow::Normal)
            }
            Statement::Return { value } => {
                let val = match value {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => Value::Undefined,
                };
                Ok(ControlFlow::Return(val))
            }
        }
    }

    /// Executes a block of statements, stopping early and propagating a
    /// RETURN the moment one is hit, rather than running the rest of the block.
    fn execute_block(&mut self, statements: &[Statement]) -> Result<ControlFlow, String> {
        for stmt in statements {
            match self.execute_statement(stmt)? {
                ControlFlow::Normal => {}
                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
            }
        }
        Ok(ControlFlow::Normal)
    }

    fn call_function(&mut self, name: &str, arguments: &[Expression]) -> Result<Value, String> {
        if let Some(func) = self.functions.get(name).cloned() {
            if arguments.len() != func.parameters.len() {
                return Err(format!(
                    "Function {} expects {} argument(s), got {}",
                    name,
                    func.parameters.len(),
                    arguments.len()
                ));
            }

            let mut arg_values = Vec::with_capacity(arguments.len());
            for arg in arguments {
                arg_values.push(self.evaluate_expression(arg)?);
            }

            let mut call_env = Environment::new();
            for (param, value) in func.parameters.iter().zip(arg_values.into_iter()) {
                call_env.set(param, value)?;
            }

            // Collect every STATIC name declared anywhere in this
            // function's body (including nested IF/FOR/WHILE branches).
            // Statics are never copied into call_env — they live
            // permanently in self.statics and are read/written directly
            // during execution via current_static_key, so the same slot
            // is shared correctly across recursive calls.
            let mut static_decls = Vec::new();
            Self::collect_static_names(&func.body, &mut static_decls);
            let static_names: Vec<String> = static_decls.iter().map(|(n, _)| n.clone()).collect();

            let caller_env = std::mem::replace(&mut self.environment, call_env);

            // Initialize each static's persistent slot the first time
            // it's ever seen, now that self.environment is the call's
            // own environment (so the initializer can reference
            // parameters if it needs to).
            for (static_name, initializer) in &static_decls {
                let key = (name.to_string(), static_name.clone());
                if !self.statics.contains_key(&key) {
                    let initial_value = self.evaluate_expression(initializer)?;
                    self.statics.insert(key, initial_value);
                }
            }

            self.static_scope_stack
                .push((name.to_string(), static_names));

            let result = self.execute_block(&func.body);

            self.static_scope_stack.pop();
            self.environment = caller_env;

            return match result? {
                ControlFlow::Return(v) => Ok(v),
                ControlFlow::Normal => Ok(Value::Undefined),
            };
        }

        if let Some(native_info) = self.native_functions.get(name).cloned() {
            let mut arg_values = Vec::with_capacity(arguments.len());
            for arg in arguments {
                arg_values.push(self.evaluate_expression(arg)?);
            }
            return (native_info.func)(&arg_values);
        }

        Err(format!("Undefined function: {}", name))
    }

    /// Recursively walks a function body (including nested IF/FOR/WHILE
    /// branches) collecting every STATIC declaration found, by name. This
    /// mirrors C's rule that a static can be lexically nested inside a
    /// conditional or loop, while its storage is fixed for the whole
    /// function regardless of whether that branch executes on a given call.
    fn collect_static_names(body: &[Statement], names: &mut Vec<(String, Expression)>) {
        for stmt in body {
            match stmt {
                Statement::StaticDeclaration { name, expression } => {
                    names.push((name.clone(), expression.clone()));
                }
                Statement::If {
                    then_branch,
                    else_if_branches,
                    else_branch,
                    ..
                } => {
                    Self::collect_static_names(then_branch, names);
                    for (_, branch) in else_if_branches {
                        Self::collect_static_names(branch, names);
                    }
                    Self::collect_static_names(else_branch, names);
                }
                Statement::ForLoop { body, .. } => {
                    Self::collect_static_names(body, names);
                }
                Statement::WhileLoop { body, .. } => {
                    Self::collect_static_names(body, names);
                }
                _ => {}
            }
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Number(n) => Ok(Value::Number(*n)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Boolean(b) => Ok(Value::Boolean(*b)),
            Expression::Identifier(name) => {
                if let Some(key) = self.current_static_key(name) {
                    return Ok(self.statics.get(&key).cloned().unwrap_or(Value::Undefined));
                }
                if self.imported_modules.contains(name) {
                    if let Some(module) = native::get_module(name) {
                        return Ok(Value::Array(module.describe()));
                    }
                }
                if let Some(value) = self.native_constants.get(name) {
                    Ok(value.clone())
                } else {
                    Ok(self.environment.get(name))
                }
            }
            Expression::ArrayAccess { name, index } => {
                let idx_val = self.evaluate_expression(index)?;
                if let Value::Number(idx) = idx_val {
                    Ok(self.environment.get_array_element(name, idx as usize))
                } else {
                    Err("Array index must be a number".to_string())
                }
            }
            Expression::ArrayLiteral(elements) => {
                let mut values = Vec::with_capacity(elements.len());
                for elem in elements {
                    values.push(self.evaluate_expression(elem)?);
                }
                Ok(Value::Array(values))
            }
            Expression::FunctionCall { name, arguments } => self.call_function(name, arguments),
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
                // Handle special cases
                if n.is_nan() {
                    return "NaN".to_string();
                }
                if n.is_infinite() {
                    if n.is_sign_positive() {
                        return "Infinity".to_string();
                    } else {
                        return "-Infinity".to_string();
                    }
                }

                // Check if it's a whole number
                if n.fract() == 0.0 {
                    return format!("{:.0}", n);
                }

                // Use Rust's debug formatting which gives a good balance
                // or use format with precision based on the value
                let formatted = format!("{:.12}", n);

                // Remove trailing zeros
                let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');

                trimmed.to_string()
            }
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Undefined => "undefined".to_string(),
        }
    }
}
