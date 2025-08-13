use crate::compat::Result;
use std::{
    collections::HashMap,
    fmt::Debug,
};

/// Closure system for NexusShell - supporting first-class functions and closures
#[derive(Debug, Clone)]
pub struct ClosureSystem {
    global_functions: HashMap<String, Function>,
    closure_registry: HashMap<String, Closure>,
    execution_context: ExecutionContext,
}

impl ClosureSystem {
    pub fn new() -> Self {
        let mut system = Self {
            global_functions: HashMap::new(),
            closure_registry: HashMap::new(),
            execution_context: ExecutionContext::new(),
        };
        
        // Register built-in functions
        system.register_builtin_functions();
        system
    }

    /// Define a function
    pub fn define_function(&mut self, name: String, function: Function) -> Result<()> {
        self.global_functions.insert(name, function);
        Ok(())
    }

    /// Create a closure that captures variables from its environment
    pub fn create_closure(&mut self, params: Vec<String>, body: Expression, captured_vars: HashMap<String, Value>) -> Result<String> {
        let closure_id = format!("closure_{}", self.closure_registry.len());
        
        let closure = Closure {
            id: closure_id.clone(),
            parameters: params,
            body,
            captured_environment: captured_vars,
            creation_context: self.execution_context.current_scope.clone(),
        };
        
        self.closure_registry.insert(closure_id.clone(), closure);
        Ok(closure_id)
    }

    /// Call a function or closure
    pub fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        // Check if it's a closure first
        if let Some(closure) = self.closure_registry.get(name).cloned() {
            return self.call_closure(&closure, args);
        }
        
        // Check if it's a global function
        if let Some(function) = self.global_functions.get(name).cloned() {
            return self.call_function(&function, args);
        }
        
    Err(crate::compat::anyhow(format!("Function or closure '{}' not found", name)))
    }

    /// Call a closure with captured environment
    fn call_closure(&mut self, closure: &Closure, args: Vec<Value>) -> Result<Value> {
        if args.len() != closure.parameters.len() {
            return Err(crate::anyhow!(
                "Closure expected {} arguments, got {}",
                closure.parameters.len(),
                args.len()
            ));
        }

        // Create new execution scope
        let mut new_scope = ExecutionScope::new();
        
        // Bind captured variables
        for (name, value) in &closure.captured_environment {
            new_scope.variables.insert(name.clone(), value.clone());
        }
        
        // Bind parameters
        for (param, arg) in closure.parameters.iter().zip(args.iter()) {
            new_scope.variables.insert(param.clone(), arg.clone());
        }
        
        // Push new scope and execute
        self.execution_context.push_scope(new_scope);
        let result = self.evaluate_expression(&closure.body);
        self.execution_context.pop_scope();
        
        result
    }

    /// Call a regular function
    fn call_function(&mut self, function: &Function, args: Vec<Value>) -> Result<Value> {
        match function {
            Function::Builtin(builtin_fn) => {
                builtin_fn.call(args)
            },
            Function::UserDefined { parameters, body } => {
                if args.len() != parameters.len() {
                    return Err(crate::anyhow!(
                        "Function expected {} arguments, got {}",
                        parameters.len(),
                        args.len()
                    ));
                }

                // Create new scope for function execution
                let mut new_scope = ExecutionScope::new();
                for (param, arg) in parameters.iter().zip(args.iter()) {
                    new_scope.variables.insert(param.clone(), arg.clone());
                }

                self.execution_context.push_scope(new_scope);
                let result = self.evaluate_expression(body);
                self.execution_context.pop_scope();

                result
            },
        }
    }

    /// Evaluate an expression
    pub fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value> {
        match expr {
            Expression::Literal(lit) => Ok(self.literal_to_value(lit)),
            
            Expression::Variable(name) => {
                self.execution_context.get_variable(name)
                    .ok_or_else(|| crate::anyhow!("Variable '{}' not found", name))
            },
            
            Expression::FunctionCall { name, args } => {
                let evaluated_args = args.iter()
                    .map(|arg| self.evaluate_expression(arg))
                    .collect::<Result<Vec<_>>>()?;
                
                self.call(name, evaluated_args)
            },
            
            Expression::Lambda { params, body } => {
                // Create a closure capturing current environment
                let captured_vars = self.execution_context.capture_environment();
                let closure_id = self.create_closure(params.clone(), (**body).clone(), captured_vars)?;
                Ok(Value::Closure(closure_id))
            },
            
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.apply_binary_op(op, &left_val, &right_val)
            },
            
            Expression::IfElse { condition, then_expr, else_expr } => {
                let condition_val = self.evaluate_expression(condition)?;
                
                if self.value_to_bool(&condition_val) {
                    self.evaluate_expression(then_expr)
                } else if let Some(else_expr) = else_expr {
                    self.evaluate_expression(else_expr)
                } else {
                    Ok(Value::Null)
                }
            },
            
            Expression::Block(statements) => {
                let mut result = Value::Null;
                for stmt in statements {
                    result = self.evaluate_expression(stmt)?;
                }
                Ok(result)
            },
            
            Expression::Assignment { var_name, value } => {
                let evaluated_value = self.evaluate_expression(value)?;
                self.execution_context.set_variable(var_name.clone(), evaluated_value.clone());
                Ok(evaluated_value)
            },
        }
    }

    /// High-order function support: map
    pub fn map_function(&mut self, list: Vec<Value>, func_name: &str) -> Result<Vec<Value>> {
        let mut results = Vec::new();
        
        for item in list {
            let result = self.call(func_name, vec![item])?;
            results.push(result);
        }
        
        Ok(results)
    }

    /// High-order function support: filter
    pub fn filter_function(&mut self, list: Vec<Value>, predicate_name: &str) -> Result<Vec<Value>> {
        let mut results = Vec::new();
        
        for item in list {
            let result = self.call(predicate_name, vec![item.clone()])?;
            if self.value_to_bool(&result) {
                results.push(item);
            }
        }
        
        Ok(results)
    }

    /// High-order function support: reduce
    pub fn reduce_function(&mut self, list: Vec<Value>, func_name: &str, initial: Value) -> Result<Value> {
        let mut accumulator = initial;
        
        for item in list {
            accumulator = self.call(func_name, vec![accumulator, item])?;
        }
        
        Ok(accumulator)
    }

    /// Register built-in functions
    fn register_builtin_functions(&mut self) {
        // Math functions
        self.global_functions.insert("add".to_string(), Function::Builtin(BuiltinFunction::new("add", |args| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                    (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                    (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
                    (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                    _ => Err(crate::anyhow!("Cannot add these types"))
                }
            } else {
                Err(crate::anyhow!("add requires exactly 2 arguments"))
            }
        })));

        self.global_functions.insert("multiply".to_string(), Function::Builtin(BuiltinFunction::new("multiply", |args| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                    (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                    (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
                    _ => Err(crate::anyhow!("Cannot multiply these types"))
                }
            } else {
                Err(crate::anyhow!("multiply requires exactly 2 arguments"))
            }
        })));

        // List functions
        self.global_functions.insert("length".to_string(), Function::Builtin(BuiltinFunction::new("length", |args| {
            if args.len() == 1 {
                match &args[0] {
                    Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
                    Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                    _ => Ok(Value::Integer(0)),
                }
            } else {
                Err(crate::anyhow!("length requires exactly 1 argument"))
            }
        })));

        // String functions
        self.global_functions.insert("uppercase".to_string(), Function::Builtin(BuiltinFunction::new("uppercase", |args| {
            if args.len() == 1 {
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.to_uppercase())),
                    _ => Err(crate::anyhow!("uppercase requires a string argument"))
                }
            } else {
                Err(crate::anyhow!("uppercase requires exactly 1 argument"))
            }
        })));
    }

    fn literal_to_value(&self, literal: &Literal) -> Value {
        match literal {
            Literal::String(s) => Value::String(s.clone()),
            Literal::Integer(i) => Value::Integer(*i),
            Literal::Float(f) => Value::Float(*f),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::Null => Value::Null,
        }
    }

    fn apply_binary_op(&self, op: &BinaryOperator, left: &Value, right: &Value) -> Result<Value> {
        match op {
            BinaryOperator::Add => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err(crate::anyhow!("Cannot add these types"))
            },
            BinaryOperator::Subtract => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(crate::anyhow!("Cannot subtract these types"))
            },
            BinaryOperator::Multiply => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
                _ => Err(crate::anyhow!("Cannot multiply these types"))
            },
            BinaryOperator::Divide => match (left, right) {
                (Value::Integer(_), Value::Integer(0)) => Err(crate::anyhow!("Division by zero")),
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Float(*a as f64 / *b as f64)),
                (Value::Float(_), Value::Float(b)) if *b == 0.0 => Err(crate::anyhow!("Division by zero")),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Integer(a), Value::Float(b)) => if *b == 0.0 { Err(crate::anyhow!("Division by zero")) } else { Ok(Value::Float(*a as f64 / b)) },
                (Value::Float(a), Value::Integer(b)) => if *b == 0 { Err(crate::anyhow!("Division by zero")) } else { Ok(Value::Float(a / *b as f64)) },
                _ => Err(crate::anyhow!("Cannot divide these types"))
            },
            BinaryOperator::Equal => Ok(Value::Boolean(left == right)),
            BinaryOperator::NotEqual => Ok(Value::Boolean(left != right)),
            BinaryOperator::LessThan => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f64) < *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a < (*b as f64))),
                _ => Err(crate::anyhow!("Cannot compare these types"))
            },
            BinaryOperator::LessEqual => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f64) <= *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a <= (*b as f64))),
                _ => Err(crate::anyhow!("Cannot compare these types"))
            },
            BinaryOperator::GreaterThan => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f64) > *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a > (*b as f64))),
                _ => Err(crate::anyhow!("Cannot compare these types"))
            },
            BinaryOperator::GreaterEqual => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f64) >= *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a >= (*b as f64))),
                _ => Err(crate::anyhow!("Cannot compare these types"))
            },
            BinaryOperator::And => Ok(Value::Boolean(self.value_to_bool(left) && self.value_to_bool(right))),
            BinaryOperator::Or  => Ok(Value::Boolean(self.value_to_bool(left) || self.value_to_bool(right))),
            // Add other operators as needed
            _ => Err(crate::anyhow!("Binary operator not implemented"))
        }
    }

    fn value_to_bool(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Null => false,
            _ => true,
        }
    }
}

/// Closure definition
#[derive(Debug, Clone)]
pub struct Closure {
    pub id: String,
    pub parameters: Vec<String>,
    pub body: Expression,
    pub captured_environment: HashMap<String, Value>,
    pub creation_context: ExecutionScope,
}

/// Function definition
#[derive(Debug, Clone)]
pub enum Function {
    Builtin(BuiltinFunction),
    UserDefined { parameters: Vec<String>, body: Expression },
}

/// Built-in function
#[derive(Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub implementation: fn(Vec<Value>) -> Result<Value>,
}

impl BuiltinFunction {
    pub fn new(name: &str, implementation: fn(Vec<Value>) -> Result<Value>) -> Self {
        Self {
            name: name.to_string(),
            implementation,
        }
    }

    pub fn call(&self, args: Vec<Value>) -> Result<Value> {
        (self.implementation)(args)
    }
}

impl Debug for BuiltinFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BuiltinFunction({})", self.name)
    }
}

/// Execution context for managing scopes
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    scope_stack: Vec<ExecutionScope>,
    current_scope: ExecutionScope,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            scope_stack: Vec::new(),
            current_scope: ExecutionScope::new(),
        }
    }

    pub fn push_scope(&mut self, scope: ExecutionScope) {
        self.scope_stack.push(std::mem::replace(&mut self.current_scope, scope));
    }

    pub fn pop_scope(&mut self) {
        if let Some(scope) = self.scope_stack.pop() {
            self.current_scope = scope;
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<Value> {
        // Look in current scope first
        if let Some(value) = self.current_scope.variables.get(name) {
            return Some(value.clone());
        }
        
        // Then look in parent scopes
        for scope in self.scope_stack.iter().rev() {
            if let Some(value) = scope.variables.get(name) {
                return Some(value.clone());
            }
        }
        
        None
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.current_scope.variables.insert(name, value);
    }

    pub fn capture_environment(&self) -> HashMap<String, Value> {
        let mut captured = HashMap::new();
        
        // Capture from all scopes
        for scope in &self.scope_stack {
            for (name, value) in &scope.variables {
                captured.insert(name.clone(), value.clone());
            }
        }
        
        // Current scope overrides
        for (name, value) in &self.current_scope.variables {
            captured.insert(name.clone(), value.clone());
        }
        
        captured
    }
}

/// Execution scope
#[derive(Debug, Clone)]
pub struct ExecutionScope {
    pub variables: HashMap<String, Value>,
}

impl ExecutionScope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

/// Expression types for closures
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    FunctionCall { name: String, args: Vec<Expression> },
    Lambda { params: Vec<String>, body: Box<Expression> },
    BinaryOp { left: Box<Expression>, op: BinaryOperator, right: Box<Expression> },
    IfElse { condition: Box<Expression>, then_expr: Box<Expression>, else_expr: Option<Box<Expression>> },
    Block(Vec<Expression>),
    Assignment { var_name: String, value: Box<Expression> },
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

/// Binary operators
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    And,
    Or,
}

/// Values in the closure system
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Closure(String), // Closure ID
    Null,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_function_call() {
        let mut system = ClosureSystem::new();
        
        let result = system.call("add", vec![Value::Integer(2), Value::Integer(3)]).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_closure_creation() {
        let mut system = ClosureSystem::new();
        
        // Create a closure that adds captured value
        let mut captured = HashMap::new();
        captured.insert("x".to_string(), Value::Integer(10));
        
        let closure_id = system.create_closure(
            vec!["y".to_string()],
            Expression::FunctionCall {
                name: "add".to_string(),
                args: vec![
                    Expression::Variable("x".to_string()),
                    Expression::Variable("y".to_string()),
                ],
            },
            captured,
        ).unwrap();
        
        let result = system.call(&closure_id, vec![Value::Integer(5)]).unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_lambda_expression() {
        let mut system = ClosureSystem::new();
        
        // Create a lambda expression
        let lambda = Expression::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expression::FunctionCall {
                name: "multiply".to_string(),
                args: vec![
                    Expression::Variable("x".to_string()),
                    Expression::Literal(Literal::Integer(2)),
                ],
            }),
        };
        
        let result = system.evaluate_expression(&lambda).unwrap();
        
        // Should return a closure ID
        if let Value::Closure(closure_id) = result {
            let doubled = system.call(&closure_id, vec![Value::Integer(21)]).unwrap();
            assert_eq!(doubled, Value::Integer(42));
        } else {
            panic!("Expected closure value");
        }
    }

    #[test]
    fn test_higher_order_functions() {
        let mut system = ClosureSystem::new();
        
        // Create a doubling closure
        let double_closure_id = system.create_closure(
            vec!["x".to_string()],
            Expression::FunctionCall {
                name: "multiply".to_string(),
                args: vec![
                    Expression::Variable("x".to_string()),
                    Expression::Literal(Literal::Integer(2)),
                ],
            },
            HashMap::new(),
        ).unwrap();
        
        // Test map function
        let numbers = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let doubled = system.map_function(numbers, &double_closure_id).unwrap();
        
        assert_eq!(doubled, vec![
            Value::Integer(2),
            Value::Integer(4),
            Value::Integer(6),
        ]);
    }
}
