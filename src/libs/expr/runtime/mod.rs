pub mod value;

use crate::libs::expr::parser::ast::{BinaryOp, ColumnRef, Expr, PipeRight, UnaryOp};
use ahash::{HashMap, HashMapExt};
use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;
use value::Value;

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Column index {0} out of bounds")]
    ColumnIndexOutOfBounds(usize),
    #[error("Column '{0}' not found")]
    ColumnNotFound(String),
    #[error("Variable '{0}' not found")]
    VariableNotFound(String),
    #[error("Global variable '{0}' not found")]
    GlobalVarNotFound(String),
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Unknown function: {0}")]
    UnknownFunction(String),
    #[error("Function '{name}': expected {expected} arguments, got {got}")]
    WrongArity {
        name: String,
        expected: usize,
        got: usize,
    },
    #[error("Underscore '_' can only be used within a pipe expression")]
    UnfillableUnderscore,
}

/// Context for expression evaluation
pub struct EvalContext<'a> {
    /// Row data as strings
    pub row: &'a [String],
    /// Optional column name to index mapping
    pub headers: Option<&'a [String]>,
    /// Variable bindings (name -> value)
    pub variables: HashMap<String, Value>,
    /// Lambda parameter bindings (name -> value)
    pub lambda_params: HashMap<String, Value>,
    /// Global variables (name -> value), shared across rows
    /// NOTE: Using Rc<RefCell> for single-threaded execution.
    /// If parallelizing in the future, change to Arc<Mutex> or use thread-local storage.
    pub globals: Rc<RefCell<HashMap<String, Value>>>,
    /// Last value from pipe expression (for underscore placeholder)
    pub last_value: Option<Value>,
}

impl<'a> EvalContext<'a> {
    pub fn new(row: &'a [String]) -> Self {
        Self {
            row,
            headers: None,
            variables: HashMap::new(),
            lambda_params: HashMap::new(),
            globals: Rc::new(RefCell::new(HashMap::new())),
            last_value: None,
        }
    }

    pub fn with_headers(row: &'a [String], headers: &'a [String]) -> Self {
        Self {
            row,
            headers: Some(headers),
            variables: HashMap::new(),
            lambda_params: HashMap::new(),
            globals: Rc::new(RefCell::new(HashMap::new())),
            last_value: None,
        }
    }

    /// Clone the context for pipeline evaluation (shares globals, clears last_value)
    pub fn clone_for_pipeline(&self) -> Self {
        Self {
            row: self.row,
            headers: self.headers,
            variables: self.variables.clone(),
            lambda_params: self.lambda_params.clone(),
            globals: Rc::clone(&self.globals),
            last_value: None,
        }
    }

    /// Set built-in global variables (called per row)
    pub fn set_builtin_globals(&self, index: i64, file: &str) {
        let mut g = self.globals.borrow_mut();
        g.insert("__index".to_string(), Value::Int(index));
        g.insert("__file".to_string(), Value::String(file.to_string()));
    }

    /// Get global variable value
    /// Returns null if not found (allows default() to provide fallback)
    fn get_global(&self, name: &str) -> Result<Value, EvalError> {
        let globals = self.globals.borrow();
        Ok(globals.get(name).cloned().unwrap_or(Value::Null))
    }

    /// Set global variable value
    fn set_global(&self, name: String, value: Value) {
        self.globals.borrow_mut().insert(name, value);
    }

    /// Get value by 1-based column index
    fn get_by_index(&self, idx: usize) -> Result<Value, EvalError> {
        let zero_based = idx - 1;
        if zero_based >= self.row.len() {
            return Err(EvalError::ColumnIndexOutOfBounds(idx));
        }
        Ok(parse_value(&self.row[zero_based]))
    }

    /// Get value by column name
    fn get_by_name(&self, name: &str) -> Result<Value, EvalError> {
        if let Some(headers) = self.headers {
            for (i, header) in headers.iter().enumerate() {
                if header == name && i < self.row.len() {
                    return Ok(parse_value(&self.row[i]));
                }
            }
        }
        Err(EvalError::ColumnNotFound(name.to_string()))
    }

    /// Get variable value
    fn get_variable(&self, name: &str) -> Result<Value, EvalError> {
        self.variables
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError::VariableNotFound(name.to_string()))
    }

    /// Set variable value
    fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Get lambda parameter value
    fn get_lambda_param(&self, name: &str) -> Result<Value, EvalError> {
        self.lambda_params
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError::VariableNotFound(name.to_string()))
    }

    /// Set lambda parameter value
    pub fn set_lambda_param(&mut self, name: String, value: Value) {
        self.lambda_params.insert(name, value);
    }

    /// Clear all lambda parameters
    pub fn clear_lambda_params(&mut self) {
        self.lambda_params.clear();
    }
}

/// Parse a string value into Value (try int, then float, then string)
fn parse_value(s: &str) -> Value {
    if s.is_empty() {
        return Value::Null;
    }

    // Try integer first
    if let Ok(i) = s.parse::<i64>() {
        return Value::Int(i);
    }

    // Then float
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }

    // Fall back to string
    Value::String(s.to_string())
}

/// Evaluate an expression in the given context
pub fn eval(expr: &Expr, ctx: &mut EvalContext) -> Result<Value, EvalError> {
    match expr {
        Expr::ColumnRef(col_ref) => match col_ref {
            ColumnRef::Index(idx) => ctx.get_by_index(*idx),
            ColumnRef::Name(name) => {
                // First check if this is a variable, then fall back to column name
                if let Ok(var_val) = ctx.get_variable(name) {
                    Ok(var_val)
                } else {
                    ctx.get_by_name(name)
                }
            }
            ColumnRef::Bare(name) => Err(EvalError::ColumnNotFound(format!(
                "Bare identifier '{}' is not allowed; use '@{}' for column references",
                name, name
            ))),
            ColumnRef::WholeRow => {
                // Join all columns with tabs
                let row_str = ctx.row.join("\t");
                Ok(Value::String(row_str))
            }
        },
        Expr::Variable(name) => ctx.get_variable(name),
        Expr::LambdaParam(name) => ctx.get_lambda_param(name),
        Expr::GlobalVar(name) => ctx.get_global(name),
        Expr::Underscore => match ctx.last_value.as_ref() {
            Some(v) => Ok(v.clone()),
            None => Err(EvalError::UnfillableUnderscore),
        },
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Float(n) => Ok(Value::Float(*n)),
        Expr::String(s) => Ok(Value::String(s.clone())),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Null => Ok(Value::Null),
        Expr::List(elements) => {
            let values: Result<Vec<Value>, EvalError> =
                elements.iter().map(|e| eval(e, ctx)).collect();
            Ok(Value::List(values?))
        }
        Expr::Unary { op, expr } => {
            let val = eval(expr, ctx)?;
            match op {
                UnaryOp::Neg => match val {
                    Value::Int(i) => Ok(Value::Int(-i)),
                    Value::Float(f) => Ok(Value::Float(-f)),
                    _ => Err(EvalError::TypeError("expected numeric".to_string())),
                },
                UnaryOp::Not => Ok(Value::Bool(!val.as_bool())),
            }
        }
        Expr::Binary { op, left, right } => {
            match op {
                // Logical operators with short-circuit evaluation
                BinaryOp::And => {
                    let left_val = eval(left, ctx)?;
                    if !left_val.as_bool() {
                        // Short-circuit: left is false, return false without evaluating right
                        Ok(Value::Bool(false))
                    } else {
                        let right_val = eval(right, ctx)?;
                        Ok(Value::Bool(right_val.as_bool()))
                    }
                }
                BinaryOp::Or => {
                    let left_val = eval(left, ctx)?;
                    if left_val.as_bool() {
                        // Short-circuit: left is true, return true without evaluating right
                        Ok(Value::Bool(true))
                    } else {
                        let right_val = eval(right, ctx)?;
                        Ok(Value::Bool(right_val.as_bool()))
                    }
                }
                // All other operators evaluate both sides first
                _ => {
                    let left_val = eval(left, ctx)?;
                    let right_val = eval(right, ctx)?;

                    match op {
                        // Arithmetic
                        BinaryOp::Add => (left_val + right_val)
                            .ok_or(EvalError::TypeError("expected numeric".to_string())),
                        BinaryOp::Sub => (left_val - right_val)
                            .ok_or(EvalError::TypeError("expected numeric".to_string())),
                        BinaryOp::Mul => (left_val * right_val)
                            .ok_or(EvalError::TypeError("expected numeric".to_string())),
                        BinaryOp::Div => {
                            (left_val / right_val).ok_or(EvalError::DivisionByZero)
                        }
                        BinaryOp::Mod => {
                            (left_val % right_val).ok_or(EvalError::DivisionByZero)
                        }
                        BinaryOp::Pow => left_val
                            .pow(&right_val)
                            .ok_or(EvalError::TypeError("expected numeric".to_string())),
                        // String concatenation
                        BinaryOp::Concat => Ok(Value::String(
                            left_val.as_string() + &right_val.as_string(),
                        )),
                        // Comparison (numeric)
                        BinaryOp::Eq => Ok(left_val.eq(&right_val)),
                        BinaryOp::Ne => Ok(left_val.ne(&right_val)),
                        BinaryOp::Lt => left_val.lt(&right_val).ok_or(
                            EvalError::TypeError("expected comparable".to_string()),
                        ),
                        BinaryOp::Le => left_val.le(&right_val).ok_or(
                            EvalError::TypeError("expected comparable".to_string()),
                        ),
                        BinaryOp::Gt => left_val.gt(&right_val).ok_or(
                            EvalError::TypeError("expected comparable".to_string()),
                        ),
                        BinaryOp::Ge => left_val.ge(&right_val).ok_or(
                            EvalError::TypeError("expected comparable".to_string()),
                        ),
                        // Comparison (string)
                        BinaryOp::StrEq => Ok(Value::Bool(
                            left_val.as_string() == right_val.as_string(),
                        )),
                        BinaryOp::StrNe => Ok(Value::Bool(
                            left_val.as_string() != right_val.as_string(),
                        )),
                        BinaryOp::StrLt => {
                            Ok(Value::Bool(left_val.as_string() < right_val.as_string()))
                        }
                        BinaryOp::StrLe => Ok(Value::Bool(
                            left_val.as_string() <= right_val.as_string(),
                        )),
                        BinaryOp::StrGt => {
                            Ok(Value::Bool(left_val.as_string() > right_val.as_string()))
                        }
                        BinaryOp::StrGe => Ok(Value::Bool(
                            left_val.as_string() >= right_val.as_string(),
                        )),
                        // Logical operators are handled above
                        BinaryOp::And | BinaryOp::Or => unreachable!(),
                    }
                }
            }
        }
        Expr::Call { name, args } => {
            // Special handling for fmt function to support %(@n) and %(var) placeholders
            if name == "fmt" {
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|arg| eval(arg, ctx))
                    .collect::<Result<Vec<_>, _>>()?;
                crate::libs::expr::functions::string::fmt_with_context(
                    &arg_values,
                    Some(ctx.row),
                    Some(&ctx.variables),
                    Some(&ctx.lambda_params),
                    Some(ctx.globals.borrow()),
                )
            } else {
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|arg| eval(arg, ctx))
                    .collect::<Result<Vec<_>, _>>()?;
                crate::libs::expr::functions::global_registry().call(name, &arg_values)
            }
        }
        Expr::MethodCall { object, name, args } => {
            // Evaluate the object first
            let obj_val = eval(object, ctx)?;
            // Build args: object as first arg, followed by method args
            let mut arg_values = vec![obj_val];
            for arg in args {
                arg_values.push(eval(arg, ctx)?);
            }
            crate::libs::expr::functions::global_registry().call(name, &arg_values)
        }
        Expr::Pipe { left, right } => {
            let left_val = eval(left, ctx)?;
            // Clone context for pipeline to propagate last_value through nested calls
            let mut pipe_ctx = ctx.clone_for_pipeline();
            pipe_ctx.last_value = Some(left_val.clone());
            eval_pipe_right(right, left_val, &mut pipe_ctx)
        }
        Expr::Bind { expr, name } => {
            let val = eval(expr, ctx)?;
            // Global variables start with "__"
            if name.starts_with("__") {
                ctx.set_global(name.clone(), val.clone());
            } else {
                ctx.set_variable(name.clone(), val.clone());
            }
            Ok(val)
        }
        Expr::Block(exprs) => {
            let mut result = Value::Null;
            for expr in exprs {
                result = eval(expr, ctx)?;
            }
            Ok(result)
        }
        Expr::Lambda { params, body } => {
            // Capture both variables and lambda parameters from the current scope
            let mut captured_vars = ctx.variables.clone();
            captured_vars.extend(ctx.lambda_params.clone());
            Ok(Value::Lambda(value::LambdaValue {
                params: params.clone(),
                body: *body.clone(),
                captured_vars,
            }))
        }
    }
}

/// Check if an expression contains an underscore placeholder (recursively)
fn contains_underscore(expr: &Expr) -> bool {
    match expr {
        Expr::Underscore => true,
        Expr::Call { args, .. } | Expr::MethodCall { args, .. } => {
            args.iter().any(contains_underscore)
        }
        Expr::List(items) => items.iter().any(contains_underscore),
        Expr::Unary { expr, .. } => contains_underscore(expr),
        Expr::Binary { left, right, .. } => {
            contains_underscore(left) || contains_underscore(right)
        }
        Expr::Pipe { left, right } => {
            contains_underscore(left) || contains_underscore_pipe_right(right)
        }
        _ => false,
    }
}

/// Check if a pipe right contains an underscore
fn contains_underscore_pipe_right(pipe_right: &PipeRight) -> bool {
    match pipe_right {
        PipeRight::Call { args, .. } | PipeRight::CallWithPlaceholder { args, .. } => {
            args.iter().any(contains_underscore)
        }
    }
}

/// Evaluate pipe right-hand side with the piped value
fn eval_pipe_right(
    pipe_right: &PipeRight,
    piped_value: Value,
    ctx: &mut EvalContext,
) -> Result<Value, EvalError> {
    match pipe_right {
        PipeRight::Call { name, args } => {
            // Check if any arg contains underscore
            let has_underscore = args.iter().any(contains_underscore);
            if has_underscore {
                // Treat as CallWithPlaceholder: replace _ with piped_value
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(eval_with_placeholder(
                        arg,
                        piped_value.clone(),
                        ctx,
                    )?);
                }
                crate::libs::expr::functions::global_registry().call(name, &arg_values)
            } else {
                // No underscore: check if function requires multiple arguments
                // For multi-arg functions, user must explicitly use _
                // For single-arg functions, auto-fill piped_value for convenience
                let registry = crate::libs::expr::functions::global_registry();
                let needs_explicit_placeholder = registry
                    .get(name)
                    .map(|info| info.min_arity > 1)
                    .unwrap_or(false);

                if needs_explicit_placeholder {
                    // Multi-arg function without _: evaluate args as-is
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(eval(arg, ctx)?);
                    }
                    registry.call(name, &arg_values)
                } else {
                    // Single-arg function: auto-fill piped_value as first arg
                    let mut arg_values = vec![piped_value];
                    for arg in args {
                        arg_values.push(eval(arg, ctx)?);
                    }
                    registry.call(name, &arg_values)
                }
            }
        }
        PipeRight::CallWithPlaceholder { name, args } => {
            // Replace each _ with piped_value, keep other args as-is
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(eval_with_placeholder(arg, piped_value.clone(), ctx)?);
            }
            crate::libs::expr::functions::global_registry().call(name, &arg_values)
        }
    }
}

/// Evaluate expression, replacing underscore with the given value
fn eval_with_placeholder(
    expr: &Expr,
    placeholder_value: Value,
    ctx: &mut EvalContext,
) -> Result<Value, EvalError> {
    match expr {
        Expr::Underscore => Ok(placeholder_value),
        Expr::Call { name, args } => {
            // Special handling for fmt function to support %(@n) and %(var) placeholders
            if name == "fmt" {
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|arg| {
                        eval_with_placeholder(arg, placeholder_value.clone(), ctx)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                crate::libs::expr::functions::string::fmt_with_context(
                    &arg_values,
                    Some(ctx.row),
                    Some(&ctx.variables),
                    Some(&ctx.lambda_params),
                    Some(ctx.globals.borrow()),
                )
            } else {
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|arg| {
                        eval_with_placeholder(arg, placeholder_value.clone(), ctx)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                crate::libs::expr::functions::global_registry().call(name, &arg_values)
            }
        }
        Expr::MethodCall { object, name, args } => {
            let obj_val = eval_with_placeholder(object, placeholder_value.clone(), ctx)?;
            let mut arg_values = vec![obj_val];
            for arg in args {
                arg_values.push(eval_with_placeholder(
                    arg,
                    placeholder_value.clone(),
                    ctx,
                )?);
            }
            crate::libs::expr::functions::global_registry().call(name, &arg_values)
        }
        Expr::List(items) => {
            let values: Result<Vec<Value>, EvalError> = items
                .iter()
                .map(|e| eval_with_placeholder(e, placeholder_value.clone(), ctx))
                .collect();
            Ok(Value::List(values?))
        }
        Expr::Unary { op, expr } => {
            let val = eval_with_placeholder(expr, placeholder_value, ctx)?;
            match op {
                UnaryOp::Neg => match val {
                    Value::Int(i) => Ok(Value::Int(-i)),
                    Value::Float(f) => Ok(Value::Float(-f)),
                    _ => Err(EvalError::TypeError("expected numeric".to_string())),
                },
                UnaryOp::Not => Ok(Value::Bool(!val.as_bool())),
            }
        }
        Expr::Binary { op, left, right } => {
            // For logical operators, we need special handling to avoid evaluating both sides
            match op {
                BinaryOp::And => {
                    let left_val =
                        eval_with_placeholder(left, placeholder_value.clone(), ctx)?;
                    if !left_val.as_bool() {
                        Ok(Value::Bool(false))
                    } else {
                        let right_val =
                            eval_with_placeholder(right, placeholder_value, ctx)?;
                        Ok(Value::Bool(right_val.as_bool()))
                    }
                }
                BinaryOp::Or => {
                    let left_val =
                        eval_with_placeholder(left, placeholder_value.clone(), ctx)?;
                    if left_val.as_bool() {
                        Ok(Value::Bool(true))
                    } else {
                        let right_val =
                            eval_with_placeholder(right, placeholder_value, ctx)?;
                        Ok(Value::Bool(right_val.as_bool()))
                    }
                }
                _ => {
                    let left_val =
                        eval_with_placeholder(left, placeholder_value.clone(), ctx)?;
                    let right_val =
                        eval_with_placeholder(right, placeholder_value, ctx)?;
                    match op {
                        BinaryOp::Add => (left_val + right_val)
                            .ok_or(EvalError::TypeError("expected numeric".to_string())),
                        BinaryOp::Sub => (left_val - right_val)
                            .ok_or(EvalError::TypeError("expected numeric".to_string())),
                        BinaryOp::Mul => (left_val * right_val)
                            .ok_or(EvalError::TypeError("expected numeric".to_string())),
                        BinaryOp::Div => {
                            (left_val / right_val).ok_or(EvalError::DivisionByZero)
                        }
                        BinaryOp::Mod => {
                            (left_val % right_val).ok_or(EvalError::DivisionByZero)
                        }
                        BinaryOp::Pow => left_val
                            .pow(&right_val)
                            .ok_or(EvalError::TypeError("expected numeric".to_string())),
                        BinaryOp::Concat => Ok(Value::String(
                            left_val.as_string() + &right_val.as_string(),
                        )),
                        BinaryOp::Eq => Ok(left_val.eq(&right_val)),
                        BinaryOp::Ne => Ok(left_val.ne(&right_val)),
                        BinaryOp::Lt => left_val.lt(&right_val).ok_or(
                            EvalError::TypeError("expected comparable".to_string()),
                        ),
                        BinaryOp::Le => left_val.le(&right_val).ok_or(
                            EvalError::TypeError("expected comparable".to_string()),
                        ),
                        BinaryOp::Gt => left_val.gt(&right_val).ok_or(
                            EvalError::TypeError("expected comparable".to_string()),
                        ),
                        BinaryOp::Ge => left_val.ge(&right_val).ok_or(
                            EvalError::TypeError("expected comparable".to_string()),
                        ),
                        BinaryOp::StrEq => Ok(Value::Bool(
                            left_val.as_string() == right_val.as_string(),
                        )),
                        BinaryOp::StrNe => Ok(Value::Bool(
                            left_val.as_string() != right_val.as_string(),
                        )),
                        BinaryOp::StrLt => {
                            Ok(Value::Bool(left_val.as_string() < right_val.as_string()))
                        }
                        BinaryOp::StrLe => Ok(Value::Bool(
                            left_val.as_string() <= right_val.as_string(),
                        )),
                        BinaryOp::StrGt => {
                            Ok(Value::Bool(left_val.as_string() > right_val.as_string()))
                        }
                        BinaryOp::StrGe => Ok(Value::Bool(
                            left_val.as_string() >= right_val.as_string(),
                        )),
                        BinaryOp::And | BinaryOp::Or => unreachable!(),
                    }
                }
            }
        }
        // For other expressions, use normal eval (they don't contain underscore)
        _ => eval(expr, ctx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::expr::parser::ast::{ColumnRef, Expr};

    #[test]
    fn test_eval_column_ref() {
        let row = vec!["10".to_string(), "20".to_string(), "hello".to_string()];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::ColumnRef(ColumnRef::Index(1));
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(10));

        let expr = Expr::ColumnRef(ColumnRef::Index(2));
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(20));

        let expr = Expr::ColumnRef(ColumnRef::Index(3));
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("hello".to_string())
        );

        // Test @0 - whole row reference
        let expr = Expr::ColumnRef(ColumnRef::WholeRow);
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("10\t20\thello".to_string())
        );
    }

    #[test]
    fn test_eval_variable() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);
        ctx.set_variable("total".to_string(), Value::Int(100));

        let expr = Expr::Variable("total".to_string());
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(100));
    }

    #[test]
    fn test_eval_list() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::List(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)]);
        match eval(&expr, &mut ctx).unwrap() {
            Value::List(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], Value::Int(1));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_string_concat() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Concat,
            left: Box::new(Expr::String("Hello".to_string())),
            right: Box::new(Expr::String(" World".to_string())),
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("Hello World".to_string())
        );
    }

    #[test]
    fn test_eval_variable_bind() {
        let row = vec!["10".to_string(), "20".to_string()];
        let mut ctx = EvalContext::new(&row);

        // @1 + @2 as @total
        let expr = Expr::Bind {
            expr: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
                right: Box::new(Expr::ColumnRef(ColumnRef::Index(2))),
            }),
            name: "total".to_string(),
        };

        // First, evaluate the bind expression
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(30));

        // Then, the variable should be accessible
        let var_expr = Expr::Variable("total".to_string());
        assert_eq!(eval(&var_expr, &mut ctx).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_eval_block() {
        let row = vec!["10".to_string(), "20".to_string()];
        let mut ctx = EvalContext::new(&row);

        // @1 as @p; @2 as @q; @p + @q
        let expr = Expr::Block(vec![
            Expr::Bind {
                expr: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
                name: "p".to_string(),
            },
            Expr::Bind {
                expr: Box::new(Expr::ColumnRef(ColumnRef::Index(2))),
                name: "q".to_string(),
            },
            Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable("p".to_string())),
                right: Box::new(Expr::Variable("q".to_string())),
            },
        ]);

        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_eval_arithmetic() {
        let row = vec!["10".to_string(), "3".to_string()];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
            right: Box::new(Expr::ColumnRef(ColumnRef::Index(2))),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(13));

        let expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
            right: Box::new(Expr::ColumnRef(ColumnRef::Index(2))),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_eval_number_literal() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Int(42);
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(42));

        let expr = Expr::Float(3.14);
        match eval(&expr, &mut ctx).unwrap() {
            Value::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_eval_comparison() {
        let row = vec!["10".to_string(), "20".to_string()];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
            right: Box::new(Expr::ColumnRef(ColumnRef::Index(2))),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));

        let expr = Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
            right: Box::new(Expr::ColumnRef(ColumnRef::Index(2))),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_logical() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Bool(true)),
            right: Box::new(Expr::Bool(false)),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(false));

        let expr = Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Bool(true)),
            right: Box::new(Expr::Bool(false)),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_unary() {
        let row = vec!["5".to_string()];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(-5));

        let expr = Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expr::Bool(true)),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_with_headers() {
        let row = vec!["Alice".to_string(), "30".to_string()];
        let headers = vec!["name".to_string(), "age".to_string()];
        let mut ctx = EvalContext::with_headers(&row, &headers);

        let expr = Expr::ColumnRef(ColumnRef::Name("name".to_string()));
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("Alice".to_string())
        );
    }

    #[test]
    fn test_eval_method_call() {
        // Test @name.trim() - method call syntax
        let row = vec!["  hello  ".to_string()];
        let headers = vec!["name".to_string()];
        let mut ctx = EvalContext::with_headers(&row, &headers);

        let expr = Expr::MethodCall {
            object: Box::new(Expr::ColumnRef(ColumnRef::Name("name".to_string()))),
            name: "trim".to_string(),
            args: vec![],
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_eval_method_call_with_args() {
        // Test @name.substr(0, 3)
        let row = vec!["hello".to_string()];
        let headers = vec!["name".to_string()];
        let mut ctx = EvalContext::with_headers(&row, &headers);

        let expr = Expr::MethodCall {
            object: Box::new(Expr::ColumnRef(ColumnRef::Name("name".to_string()))),
            name: "substr".to_string(),
            args: vec![Expr::Int(0), Expr::Int(3)],
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("hel".to_string())
        );
    }

    #[test]
    fn test_eval_method_chain() {
        // Test @name.trim().upper()
        let row = vec!["  hello  ".to_string()];
        let headers = vec!["name".to_string()];
        let mut ctx = EvalContext::with_headers(&row, &headers);

        let expr = Expr::MethodCall {
            object: Box::new(Expr::MethodCall {
                object: Box::new(Expr::ColumnRef(ColumnRef::Name("name".to_string()))),
                name: "trim".to_string(),
                args: vec![],
            }),
            name: "upper".to_string(),
            args: vec![],
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("HELLO".to_string())
        );
    }

    #[test]
    fn test_eval_lambda() {
        // Test lambda expression: x => x + 1
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::LambdaParam("x".to_string())),
                right: Box::new(Expr::Int(1)),
            }),
        };

        // Evaluate lambda should return Lambda value
        let result = eval(&expr, &mut ctx).unwrap();
        match result {
            Value::Lambda(lambda) => {
                assert_eq!(lambda.params, vec!["x"]);
            }
            _ => panic!("Expected Lambda value, got {:?}", result),
        }
    }

    #[test]
    fn test_eval_lambda_param() {
        // Test lambda parameter resolution
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);
        ctx.lambda_params.insert("x".to_string(), Value::Int(5));

        let expr = Expr::LambdaParam("x".to_string());
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_column_index_out_of_bounds() {
        let row = vec!["10".to_string(), "20".to_string()];
        let mut ctx = EvalContext::new(&row);

        // Try to access column 3 when only 2 exist
        let expr = Expr::ColumnRef(ColumnRef::Index(3));
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Column index 3 out of bounds"));
    }

    #[test]
    fn test_column_name_not_found() {
        let row = vec!["10".to_string(), "20".to_string()];
        let headers = vec!["a".to_string(), "b".to_string()];
        let mut ctx = EvalContext::with_headers(&row, &headers);

        // Try to access non-existent column
        let expr = Expr::ColumnRef(ColumnRef::Name("c".to_string()));
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Column 'c' not found"));
    }

    #[test]
    fn test_variable_not_found() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // Try to access non-existent variable
        let expr = Expr::Variable("nonexistent".to_string());
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Variable 'nonexistent' not found"));
    }

    #[test]
    fn test_lambda_param_not_found() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // Try to access non-existent lambda param
        let expr = Expr::LambdaParam("nonexistent".to_string());
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Variable 'nonexistent' not found"));
    }

    #[test]
    fn test_unary_neg_non_numeric() {
        let row = vec!["hello".to_string()];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
        };
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected numeric"));
    }

    #[test]
    fn test_arithmetic_type_error() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // Add non-numeric values
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::String("a".to_string())),
            right: Box::new(Expr::String("b".to_string())),
        };
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected numeric"));
    }

    #[test]
    fn test_division_by_zero_error() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Int(0)),
        };
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }

    #[test]
    fn test_modulo_by_zero_error() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Mod,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Int(0)),
        };
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }

    #[test]
    fn test_comparison_type_error() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // Compare non-comparable values
        let expr = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::String("a".to_string())),
            right: Box::new(Expr::Int(1)),
        };
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expected comparable"));
    }

    #[test]
    fn test_string_comparison_operators() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // StrEq
        let expr = Expr::Binary {
            op: BinaryOp::StrEq,
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(Expr::String("hello".to_string())),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));

        // StrNe
        let expr = Expr::Binary {
            op: BinaryOp::StrNe,
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(Expr::String("world".to_string())),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));

        // StrLt
        let expr = Expr::Binary {
            op: BinaryOp::StrLt,
            left: Box::new(Expr::String("apple".to_string())),
            right: Box::new(Expr::String("banana".to_string())),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));

        // StrLe
        let expr = Expr::Binary {
            op: BinaryOp::StrLe,
            left: Box::new(Expr::String("apple".to_string())),
            right: Box::new(Expr::String("apple".to_string())),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));

        // StrGt
        let expr = Expr::Binary {
            op: BinaryOp::StrGt,
            left: Box::new(Expr::String("banana".to_string())),
            right: Box::new(Expr::String("apple".to_string())),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));

        // StrGe
        let expr = Expr::Binary {
            op: BinaryOp::StrGe,
            left: Box::new(Expr::String("apple".to_string())),
            right: Box::new(Expr::String("apple".to_string())),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_subtraction() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Int(3)),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(7));
    }

    #[test]
    fn test_power_operation() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Pow,
            left: Box::new(Expr::Int(2)),
            right: Box::new(Expr::Int(3)),
        };
        match eval(&expr, &mut ctx).unwrap() {
            Value::Float(f) => assert!((f - 8.0).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_le_ge_comparison() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // Le
        let expr = Expr::Binary {
            op: BinaryOp::Le,
            left: Box::new(Expr::Int(5)),
            right: Box::new(Expr::Int(5)),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));

        // Ge
        let expr = Expr::Binary {
            op: BinaryOp::Ge,
            left: Box::new(Expr::Int(5)),
            right: Box::new(Expr::Int(5)),
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_pipe_operation() {
        let row = vec!["  hello  ".to_string()];
        let headers = vec!["name".to_string()];
        let mut ctx = EvalContext::with_headers(&row, &headers);

        // Test @name |> trim()
        let expr = Expr::Pipe {
            left: Box::new(Expr::ColumnRef(ColumnRef::Name("name".to_string()))),
            right: Box::new(PipeRight::Call {
                name: "trim".to_string(),
                args: vec![],
            }),
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_pipe_with_placeholder() {
        let row = vec!["hello world".to_string()];
        let headers = vec!["text".to_string()];
        let mut ctx = EvalContext::with_headers(&row, &headers);

        // Test @text |> replace(_, "world", "Rust")
        let expr = Expr::Pipe {
            left: Box::new(Expr::ColumnRef(ColumnRef::Name("text".to_string()))),
            right: Box::new(PipeRight::CallWithPlaceholder {
                name: "replace".to_string(),
                args: vec![
                    Expr::Underscore,
                    Expr::String("world".to_string()),
                    Expr::String("Rust".to_string()),
                ],
            }),
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("hello Rust".to_string())
        );
    }

    #[test]
    fn test_pipe_with_explicit_placeholder() {
        // Test pipe with explicit _ placeholder: [1,2,3] | join(_, ",")
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Pipe {
            left: Box::new(Expr::List(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)])),
            right: Box::new(PipeRight::CallWithPlaceholder {
                name: "join".to_string(),
                args: vec![
                    Expr::Underscore, // Explicit placeholder
                    Expr::String(",".to_string()),
                ],
            }),
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("1,2,3".to_string())
        );
    }

    #[test]
    fn test_pipe_with_nested_placeholder() {
        // Test nested function with _: "hello" | print(substr(_, 1, 2))
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Pipe {
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(PipeRight::Call {
                name: "print".to_string(),
                args: vec![Expr::Call {
                    name: "substr".to_string(),
                    args: vec![Expr::Underscore, Expr::Int(1), Expr::Int(2)],
                }],
            }),
        };
        // print returns its last argument, which is the result of substr
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("el".to_string())
        );
    }

    #[test]
    fn test_underscore_placeholder_basic() {
        // Test basic underscore usage: "hello" | upper()
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Pipe {
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(PipeRight::Call {
                name: "upper".to_string(),
                args: vec![Expr::Underscore],
            }),
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("HELLO".to_string())
        );
    }

    #[test]
    fn test_underscore_placeholder_with_position() {
        // Test underscore in non-first position: "hello" | replace(_, "l", "L")
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Pipe {
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(PipeRight::Call {
                name: "replace".to_string(),
                args: vec![
                    Expr::Underscore,
                    Expr::String("l".to_string()),
                    Expr::String("L".to_string()),
                ],
            }),
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("heLLo".to_string())
        );
    }

    #[test]
    fn test_underscore_placeholder_chained() {
        // Test chained pipes with underscore: "hello" | upper() | substr(_, 1, 3)
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Pipe {
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(PipeRight::Call {
                name: "upper".to_string(),
                args: vec![Expr::Underscore],
            }),
        };
        let result = eval(&expr, &mut ctx).unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));

        // Now pipe to substr
        let expr2 = Expr::Pipe {
            left: Box::new(Expr::String("HELLO".to_string())),
            right: Box::new(PipeRight::Call {
                name: "substr".to_string(),
                args: vec![Expr::Underscore, Expr::Int(1), Expr::Int(3)],
            }),
        };
        assert_eq!(
            eval(&expr2, &mut ctx).unwrap(),
            Value::String("ELL".to_string())
        );
    }

    #[test]
    fn test_underscore_placeholder_without_pipe() {
        // Underscore without pipe should error
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let result = eval(&Expr::Underscore, &mut ctx);
        assert!(result.is_err(), "Underscore without pipe should error");
    }

    #[test]
    fn test_single_arg_function_without_underscore() {
        // Single-arg functions can omit _: "hello" | upper()
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Pipe {
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(PipeRight::Call {
                name: "upper".to_string(),
                args: vec![],
            }),
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("HELLO".to_string())
        );
    }

    #[test]
    fn test_multi_arg_function_without_underscore_errors() {
        // Multi-arg functions without _ should error
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Pipe {
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(PipeRight::Call {
                name: "substr".to_string(),
                args: vec![Expr::Int(1), Expr::Int(2)],
            }),
        };
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err(), "Multi-arg function without _ should error");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("expected 3 arguments"),
            "Error should mention expected argument count: {}",
            err_msg
        );
    }

    #[test]
    fn test_multi_arg_function_with_underscore() {
        // Multi-arg functions with _ should work
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Pipe {
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(PipeRight::Call {
                name: "substr".to_string(),
                args: vec![Expr::Underscore, Expr::Int(1), Expr::Int(2)],
            }),
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("el".to_string())
        );
    }

    #[test]
    fn test_multiple_underscore_placeholders() {
        // Multiple _ in same call should all get the piped value
        // "hello" | concat(substr(_, 1, 2), substr(_, 2, 2))
        // Should produce "el" ++ "ll" = "elll"
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // Use replace(_, "l", "L") and replace(_, "o", "O") to test multiple _
        // "hello" -> "heLLo" and "heLLo" -> "heLLO"
        let expr = Expr::Pipe {
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(PipeRight::Call {
                name: "replace".to_string(),
                args: vec![
                    Expr::Call {
                        name: "replace".to_string(),
                        args: vec![
                            Expr::Underscore,
                            Expr::String("l".to_string()),
                            Expr::String("L".to_string()),
                        ],
                    },
                    Expr::String("o".to_string()),
                    Expr::String("O".to_string()),
                ],
            }),
        };
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("heLLO".to_string())
        );
    }

    #[test]
    fn test_variable_fallback_to_column() {
        // When a variable is not found, it should fall back to column name lookup
        let row = vec!["100".to_string()];
        let headers = vec!["value".to_string()];
        let mut ctx = EvalContext::with_headers(&row, &headers);

        // This should find the column "value" since no variable named "value" exists
        let expr = Expr::ColumnRef(ColumnRef::Name("value".to_string()));
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(100));
    }

    #[test]
    fn test_eval_null_literal() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Null;
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Null);
    }

    #[test]
    fn test_eval_bool_literal() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Bool(true);
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));

        let expr = Expr::Bool(false);
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_string_literal() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::String("hello".to_string());
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_clear_lambda_params() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        ctx.set_lambda_param("x".to_string(), Value::Int(1));
        ctx.set_lambda_param("y".to_string(), Value::Int(2));
        assert_eq!(ctx.lambda_params.len(), 2);

        ctx.clear_lambda_params();
        assert_eq!(ctx.lambda_params.len(), 0);
    }

    #[test]
    fn test_eval_call() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // Test abs(-5)
        let expr = Expr::Call {
            name: "abs".to_string(),
            args: vec![Expr::Int(-5)],
        };
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_eval_empty_block() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // Empty block should return Null
        let expr = Expr::Block(vec![]);
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Null);
    }

    #[test]
    fn test_eval_lambda_captures_variables() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);
        ctx.set_variable("outer".to_string(), Value::Int(42));

        // Lambda should capture outer variables
        let expr = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable("outer".to_string())),
                right: Box::new(Expr::LambdaParam("x".to_string())),
            }),
        };

        let result = eval(&expr, &mut ctx).unwrap();
        match result {
            Value::Lambda(lambda) => {
                assert_eq!(lambda.params, vec!["x"]);
                // Check that outer variable was captured
                assert_eq!(lambda.captured_vars.get("outer"), Some(&Value::Int(42)));
            }
            _ => panic!("Expected Lambda"),
        }
    }

    // Integration tests moved from src/libs/expr/tests/basic.rs
    #[test]
    fn test_integration_simple_column_reference() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec!["10".to_string(), "20".to_string(), "30".to_string()];
        assert_eq!(eval_expr("@1", &r, None).unwrap().to_string(), "10");
        assert_eq!(eval_expr("@2", &r, None).unwrap().to_string(), "20");
        assert_eq!(eval_expr("@3", &r, None).unwrap().to_string(), "30");
    }

    #[test]
    fn test_integration_basic_arithmetic() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec!["10".to_string(), "5".to_string()];
        assert_eq!(eval_expr("@1 + @2", &r, None).unwrap().to_string(), "15");
        assert_eq!(eval_expr("@1 - @2", &r, None).unwrap().to_string(), "5");
        assert_eq!(eval_expr("@1 * @2", &r, None).unwrap().to_string(), "50");
        assert_eq!(eval_expr("@1 / @2", &r, None).unwrap().to_string(), "2");
    }

    #[test]
    fn test_integration_operator_precedence() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec!["10".to_string(), "3".to_string(), "2".to_string()];
        assert_eq!(
            eval_expr("@1 + @2 * @3", &r, None).unwrap().to_string(),
            "16"
        );
        assert_eq!(
            eval_expr("(@1 + @2) * @3", &r, None).unwrap().to_string(),
            "26"
        );
    }

    #[test]
    fn test_integration_column_reference_by_name() {
        use crate::libs::expr::eval_expr;
        let h: Vec<String> = vec!["price".to_string(), "quantity".to_string()];
        let r: Vec<String> = vec!["100".to_string(), "5".to_string()];
        assert_eq!(
            eval_expr("@price", &r, Some(&h)).unwrap().to_string(),
            "100"
        );
        assert_eq!(
            eval_expr("@quantity", &r, Some(&h)).unwrap().to_string(),
            "5"
        );
        assert_eq!(
            eval_expr("@price * @quantity", &r, Some(&h))
                .unwrap()
                .to_string(),
            "500"
        );
    }

    #[test]
    fn test_integration_number_literals() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];
        assert_eq!(eval_expr("42", &r, None).unwrap().to_string(), "42");
        assert_eq!(eval_expr("3.14", &r, None).unwrap().to_string(), "3.14");
    }

    #[test]
    fn test_integration_string_literals() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];
        assert_eq!(
            eval_expr("\"hello\"", &r, None).unwrap().to_string(),
            "hello"
        );
        assert_eq!(eval_expr("'world'", &r, None).unwrap().to_string(), "world");
    }

    #[test]
    fn test_integration_boolean_literals() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];
        assert_eq!(eval_expr("true", &r, None).unwrap().to_string(), "true");
        assert_eq!(eval_expr("false", &r, None).unwrap().to_string(), "false");
    }

    #[test]
    fn test_integration_null_literal() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];
        assert_eq!(eval_expr("null", &r, None).unwrap().to_string(), "null");
    }

    #[test]
    fn test_integration_comparison_operators() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec!["10".to_string(), "5".to_string()];
        assert_eq!(eval_expr("@1 > @2", &r, None).unwrap().to_string(), "true");
        assert_eq!(eval_expr("@1 < @2", &r, None).unwrap().to_string(), "false");
        assert_eq!(
            eval_expr("@1 == @2", &r, None).unwrap().to_string(),
            "false"
        );
        assert_eq!(eval_expr("@1 != @2", &r, None).unwrap().to_string(), "true");
    }

    #[test]
    fn test_integration_logical_operators() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];
        assert_eq!(
            eval_expr("true and true", &r, None).unwrap().to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("true and false", &r, None).unwrap().to_string(),
            "false"
        );
        assert_eq!(
            eval_expr("true or false", &r, None).unwrap().to_string(),
            "true"
        );
    }

    #[test]
    fn test_integration_unary_operators() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];
        assert_eq!(eval_expr("-5", &r, None).unwrap().to_string(), "-5");
        assert_eq!(
            eval_expr("not true", &r, None).unwrap().to_string(),
            "false"
        );
    }

    #[test]
    fn test_integration_power_operator() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];
        assert_eq!(eval_expr("2 ** 3", &r, None).unwrap().to_string(), "8");
    }

    #[test]
    fn test_integration_modulo_operator() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];
        assert_eq!(eval_expr("10 % 3", &r, None).unwrap().to_string(), "1");
    }

    #[test]
    fn test_integration_string_comparison_operators() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec!["alice".to_string(), "bob".to_string()];
        assert_eq!(eval_expr("@1 eq @1", &r, None).unwrap().to_string(), "true");
        assert_eq!(eval_expr("@1 lt @2", &r, None).unwrap().to_string(), "true");
        assert_eq!(
            eval_expr("@1 gt @2", &r, None).unwrap().to_string(),
            "false"
        );
    }

    // Error handling tests moved from src/libs/expr/tests/errors.rs
    #[test]
    fn test_integration_eval_column_out_of_bounds() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec!["10".to_string()];
        assert!(eval_expr("@2", &r, None).is_err());
    }

    #[test]
    fn test_integration_eval_unknown_column_name() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec!["10".to_string()];
        let h: Vec<String> = vec!["a".to_string()];
        assert!(eval_expr("@unknown", &r, Some(&h)).is_err());
    }

    #[test]
    fn test_integration_eval_division_by_zero() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec!["10".to_string(), "0".to_string()];
        let result = eval_expr("@1 / @2", &r, None);
        assert!(result.is_err());
    }

    // Function-related integration tests moved from src/libs/expr/tests/functions.rs
    #[test]
    fn test_integration_nested_function_calls() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec!["  hello  ".to_string()];
        assert_eq!(
            eval_expr("upper(trim(@1))", &r, None).unwrap().to_string(),
            "HELLO"
        );
    }

    #[test]
    fn test_integration_function_with_column_ref() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec!["  alice  ".to_string()];
        let h: Vec<String> = vec!["name".to_string()];
        assert_eq!(
            eval_expr("upper(trim(@name))", &r, Some(&h))
                .unwrap()
                .to_string(),
            "ALICE"
        );
    }

    #[test]
    fn test_integration_unknown_function_error() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];
        let result = eval_expr("unknown(1)", &r, None);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Unknown function"));
    }

    #[test]
    fn test_integration_wrong_arity_error() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];
        let result = eval_expr("trim()", &r, None);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("expected"));
    }

    // Short-circuit evaluation tests
    #[test]
    fn test_and_short_circuit() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];

        // false && anything should be false without evaluating right side
        // Using 1/0 which would cause division by zero if evaluated
        let result = eval_expr("false and (1 / 0)", &r, None);
        assert!(
            result.is_ok(),
            "Short-circuit should prevent division by zero"
        );
        assert_eq!(result.unwrap().to_string(), "false");

        // true && true should evaluate both sides
        let result = eval_expr("true and true", &r, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "true");

        // true && false should evaluate both sides and return false
        let result = eval_expr("true and false", &r, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "false");
    }

    #[test]
    fn test_or_short_circuit() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];

        // true || anything should be true without evaluating right side
        // Using 1/0 which would cause division by zero if evaluated
        let result = eval_expr("true or (1 / 0)", &r, None);
        assert!(
            result.is_ok(),
            "Short-circuit should prevent division by zero"
        );
        assert_eq!(result.unwrap().to_string(), "true");

        // false || false should evaluate both sides
        let result = eval_expr("false or false", &r, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "false");

        // false || true should evaluate both sides and return true
        let result = eval_expr("false or true", &r, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_short_circuit_with_null() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];

        // false && null should be false (null not evaluated)
        let result = eval_expr("false and null", &r, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "false");

        // true || null should be true (null not evaluated)
        let result = eval_expr("true or null", &r, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_short_circuit_chained() {
        use crate::libs::expr::eval_expr;
        let r: Vec<String> = vec![];

        // Chained and: false && (1/0) && (2/0) should short-circuit at first
        let result = eval_expr("false and (1 / 0) and (2 / 0)", &r, None);
        assert!(
            result.is_ok(),
            "Short-circuit should prevent division by zero"
        );
        assert_eq!(result.unwrap().to_string(), "false");

        // Chained or: true || (1/0) || (2/0) should short-circuit at first
        let result = eval_expr("true or (1 / 0) or (2 / 0)", &r, None);
        assert!(
            result.is_ok(),
            "Short-circuit should prevent division by zero"
        );
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_eval_context_with_headers() {
        let row = vec!["Alice".to_string(), "30".to_string()];
        let headers = vec!["name".to_string(), "age".to_string()];
        let ctx = EvalContext::with_headers(&row, &headers);

        assert_eq!(ctx.row.len(), 2);
        assert!(ctx.headers.is_some());
        assert_eq!(ctx.headers.unwrap().len(), 2);
    }

    #[test]
    fn test_eval_context_new() {
        let row = vec!["test".to_string()];
        let ctx = EvalContext::new(&row);

        assert_eq!(ctx.row.len(), 1);
        assert!(ctx.headers.is_none());
        assert!(ctx.variables.is_empty());
        assert!(ctx.lambda_params.is_empty());
    }

    #[test]
    fn test_eval_context_set_get_variable() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        ctx.set_variable("test".to_string(), Value::Int(42));
        assert_eq!(ctx.get_variable("test").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eval_context_set_get_lambda_param() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        ctx.set_lambda_param("x".to_string(), Value::Int(10));
        assert_eq!(ctx.get_lambda_param("x").unwrap(), Value::Int(10));
    }

    #[test]
    fn test_eval_context_clear_lambda_params() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        ctx.set_lambda_param("x".to_string(), Value::Int(1));
        ctx.set_lambda_param("y".to_string(), Value::Int(2));
        assert_eq!(ctx.lambda_params.len(), 2);

        ctx.clear_lambda_params();
        assert!(ctx.lambda_params.is_empty());
    }

    #[test]
    fn test_eval_context_get_by_index() {
        let row = vec!["10".to_string(), "20".to_string(), "30".to_string()];
        let ctx = EvalContext::new(&row);

        assert_eq!(ctx.get_by_index(1).unwrap(), Value::Int(10));
        assert_eq!(ctx.get_by_index(2).unwrap(), Value::Int(20));
        assert_eq!(ctx.get_by_index(3).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_eval_context_get_by_index_out_of_bounds() {
        let row = vec!["10".to_string()];
        let ctx = EvalContext::new(&row);

        assert!(matches!(
            ctx.get_by_index(2),
            Err(EvalError::ColumnIndexOutOfBounds(2))
        ));
    }

    #[test]
    fn test_eval_context_get_by_name() {
        let row = vec!["Alice".to_string(), "30".to_string()];
        let headers = vec!["name".to_string(), "age".to_string()];
        let ctx = EvalContext::with_headers(&row, &headers);

        assert_eq!(
            ctx.get_by_name("name").unwrap(),
            Value::String("Alice".to_string())
        );
        assert_eq!(ctx.get_by_name("age").unwrap(), Value::Int(30));
    }

    #[test]
    fn test_eval_context_get_by_name_not_found() {
        let row = vec!["Alice".to_string()];
        let headers = vec!["name".to_string()];
        let ctx = EvalContext::with_headers(&row, &headers);

        assert!(matches!(
            ctx.get_by_name("unknown"),
            Err(EvalError::ColumnNotFound(name)) if name == "unknown"
        ));
    }

    #[test]
    fn test_eval_context_get_by_name_no_headers() {
        let row = vec!["Alice".to_string()];
        let ctx = EvalContext::new(&row);

        assert!(matches!(
            ctx.get_by_name("name"),
            Err(EvalError::ColumnNotFound(name)) if name == "name"
        ));
    }

    #[test]
    fn test_parse_value_empty() {
        assert_eq!(parse_value(""), Value::Null);
    }

    #[test]
    fn test_parse_value_integer() {
        assert_eq!(parse_value("42"), Value::Int(42));
        assert_eq!(parse_value("-100"), Value::Int(-100));
        assert_eq!(parse_value("0"), Value::Int(0));
    }

    #[test]
    fn test_parse_value_float() {
        assert_eq!(parse_value("3.14"), Value::Float(3.14));
        assert_eq!(parse_value("-2.5"), Value::Float(-2.5));
        assert_eq!(parse_value("0.0"), Value::Float(0.0));
    }

    #[test]
    fn test_parse_value_string() {
        assert_eq!(parse_value("hello"), Value::String("hello".to_string()));
        assert_eq!(parse_value("123abc"), Value::String("123abc".to_string()));
    }

    #[test]
    fn test_eval_pipe_right_call() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let pipe_right = PipeRight::Call {
            name: "abs".to_string(),
            args: vec![],
        };

        let result = eval_pipe_right(&pipe_right, Value::Int(-5), &mut ctx);
        assert_eq!(result.unwrap(), Value::Int(5));
    }

    #[test]
    fn test_eval_pipe_right_call_with_placeholder() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let pipe_right = PipeRight::CallWithPlaceholder {
            name: "replace".to_string(),
            args: vec![
                Expr::Underscore,
                Expr::String("old".to_string()),
                Expr::String("new".to_string()),
            ],
        };

        let result = eval_pipe_right(
            &pipe_right,
            Value::String("hello old world".to_string()),
            &mut ctx,
        );
        assert_eq!(
            result.unwrap(),
            Value::String("hello new world".to_string())
        );
    }

    #[test]
    fn test_eval_call_with_args() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Call {
            name: "max".to_string(),
            args: vec![Expr::Int(1), Expr::Int(5), Expr::Int(3)],
        };

        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_eval_method_call_with_object() {
        let row = vec!["hello".to_string()];
        let headers = vec!["text".to_string()];
        let mut ctx = EvalContext::with_headers(&row, &headers);

        let expr = Expr::MethodCall {
            object: Box::new(Expr::ColumnRef(ColumnRef::Name("text".to_string()))),
            name: "upper".to_string(),
            args: vec![],
        };

        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("HELLO".to_string())
        );
    }

    #[test]
    fn test_eval_lambda_with_captured_vars() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);
        ctx.set_variable("outer".to_string(), Value::Int(10));

        let expr = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable("outer".to_string())),
                right: Box::new(Expr::LambdaParam("x".to_string())),
            }),
        };

        let result = eval(&expr, &mut ctx).unwrap();
        match result {
            Value::Lambda(lambda) => {
                assert_eq!(lambda.captured_vars.get("outer"), Some(&Value::Int(10)));
            }
            _ => panic!("Expected Lambda"),
        }
    }

    #[test]
    fn test_eval_lambda_param_in_body() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);
        ctx.set_lambda_param("x".to_string(), Value::Int(5));

        let expr = Expr::LambdaParam("x".to_string());
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_eval_variable_not_found() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Variable("nonexistent".to_string());
        assert!(matches!(
            eval(&expr, &mut ctx),
            Err(EvalError::VariableNotFound(name)) if name == "nonexistent"
        ));
    }

    #[test]
    fn test_eval_unary_neg_float() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::Float(3.14)),
        };

        match eval(&expr, &mut ctx).unwrap() {
            Value::Float(f) => assert!((f + 3.14).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_eval_unary_not_non_bool() {
        let row = vec!["hello".to_string()];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
        };

        // Non-empty string is truthy, so not should be false
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_list_with_expressions() {
        let row = vec!["10".to_string(), "20".to_string()];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::List(vec![
            Expr::ColumnRef(ColumnRef::Index(1)),
            Expr::ColumnRef(ColumnRef::Index(2)),
            Expr::Int(30),
        ]);

        match eval(&expr, &mut ctx).unwrap() {
            Value::List(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], Value::Int(10));
                assert_eq!(values[1], Value::Int(20));
                assert_eq!(values[2], Value::Int(30));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_nested_list() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::List(vec![
            Expr::List(vec![Expr::Int(1), Expr::Int(2)]),
            Expr::List(vec![Expr::Int(3), Expr::Int(4)]),
        ]);

        match eval(&expr, &mut ctx).unwrap() {
            Value::List(values) => {
                assert_eq!(values.len(), 2);
                match &values[0] {
                    Value::List(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(inner[0], Value::Int(1));
                    }
                    _ => panic!("Expected nested List"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_block_single_expr() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Block(vec![Expr::Int(42)]);
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eval_block_multiple_binds() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Block(vec![
            Expr::Bind {
                expr: Box::new(Expr::Int(1)),
                name: "a".to_string(),
            },
            Expr::Bind {
                expr: Box::new(Expr::Int(2)),
                name: "b".to_string(),
            },
            Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable("a".to_string())),
                right: Box::new(Expr::Variable("b".to_string())),
            },
        ]);

        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(3));
    }

    #[test]
    fn test_eval_comparison_with_floats() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::Float(1.5)),
            right: Box::new(Expr::Float(2.5)),
        };

        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_string_comparison() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::StrEq,
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(Expr::String("hello".to_string())),
        };

        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_concat() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Concat,
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(Expr::String(" world".to_string())),
        };

        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("hello world".to_string())
        );
    }

    #[test]
    fn test_eval_power() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Pow,
            left: Box::new(Expr::Int(2)),
            right: Box::new(Expr::Int(3)),
        };

        match eval(&expr, &mut ctx).unwrap() {
            Value::Float(f) => assert!((f - 8.0).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_eval_modulo() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Mod,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Int(3)),
        };

        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(1));
    }

    #[test]
    fn test_eval_division_by_zero() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Int(0)),
        };

        assert!(matches!(
            eval(&expr, &mut ctx),
            Err(EvalError::DivisionByZero)
        ));
    }

    #[test]
    fn test_eval_unknown_function() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Call {
            name: "nonexistent_function".to_string(),
            args: vec![],
        };

        assert!(matches!(
            eval(&expr, &mut ctx),
            Err(EvalError::UnknownFunction(name)) if name == "nonexistent_function"
        ));
    }

    #[test]
    fn test_eval_wrong_arity() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // abs() expects exactly 1 argument
        let expr = Expr::Call {
            name: "abs".to_string(),
            args: vec![],
        };

        assert!(matches!(
            eval(&expr, &mut ctx),
            Err(EvalError::WrongArity { name, expected: 1, got: 0 }) if name == "abs"
        ));
    }

    #[test]
    fn test_eval_expr_helper() {
        use crate::libs::expr::eval_expr;

        let row = vec!["10".to_string(), "20".to_string()];
        let headers = vec!["a".to_string(), "b".to_string()];

        assert_eq!(eval_expr("@1 + @2", &row, None).unwrap().to_string(), "30");
        assert_eq!(
            eval_expr("@a + @b", &row, Some(&headers))
                .unwrap()
                .to_string(),
            "30"
        );
    }

    #[test]
    fn test_eval_expr_with_error() {
        use crate::libs::expr::eval_expr;

        let row = vec!["10".to_string()];

        assert!(eval_expr("@2", &row, None).is_err());
        assert!(eval_expr("unknown()", &row, None).is_err());
    }

    #[test]
    fn test_eval_pipe_with_method_call() {
        use crate::libs::expr::eval_expr;

        let row = vec!["  hello  ".to_string()];
        let headers = vec!["name".to_string()];

        let result = eval_expr("@name | trim() | upper()", &row, Some(&headers));
        assert_eq!(result.unwrap().to_string(), "HELLO");
    }

    #[test]
    fn test_eval_complex_arithmetic() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        // Test operator precedence
        assert_eq!(
            eval_expr("2 + 3 * 4", &row, None).unwrap().to_string(),
            "14"
        );
        assert_eq!(
            eval_expr("(2 + 3) * 4", &row, None).unwrap().to_string(),
            "20"
        );
        assert_eq!(
            eval_expr("10 - 3 - 2", &row, None).unwrap().to_string(),
            "5"
        );
    }

    #[test]
    fn test_eval_float_arithmetic() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        let result = eval_expr("3.14 + 2.86", &row, None).unwrap();
        match result {
            Value::Float(f) => assert!((f - 6.0).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_eval_mixed_type_arithmetic() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        // Int + Float = Float
        let result = eval_expr("10 + 2.5", &row, None).unwrap();
        match result {
            Value::Float(f) => assert!((f - 12.5).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_eval_null_handling() {
        use crate::libs::expr::eval_expr;

        let row = vec!["".to_string()];

        // Empty field is null
        assert_eq!(
            eval_expr("@1 == null", &row, None).unwrap().to_string(),
            "true"
        );
        assert_eq!(eval_expr("not @1", &row, None).unwrap().to_string(), "true");
    }

    #[test]
    fn test_eval_boolean_operations() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("true and true", &row, None).unwrap().to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("true and false", &row, None).unwrap().to_string(),
            "false"
        );
        assert_eq!(
            eval_expr("true or false", &row, None).unwrap().to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("false or false", &row, None).unwrap().to_string(),
            "false"
        );
        assert_eq!(
            eval_expr("not true", &row, None).unwrap().to_string(),
            "false"
        );
        assert_eq!(
            eval_expr("not false", &row, None).unwrap().to_string(),
            "true"
        );
    }

    #[test]
    fn test_eval_comparison_operators() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(eval_expr("5 == 5", &row, None).unwrap().to_string(), "true");
        assert_eq!(eval_expr("5 != 3", &row, None).unwrap().to_string(), "true");
        assert_eq!(eval_expr("3 < 5", &row, None).unwrap().to_string(), "true");
        assert_eq!(eval_expr("5 <= 5", &row, None).unwrap().to_string(), "true");
        assert_eq!(eval_expr("5 > 3", &row, None).unwrap().to_string(), "true");
        assert_eq!(eval_expr("5 >= 5", &row, None).unwrap().to_string(), "true");
    }

    #[test]
    fn test_eval_string_operations() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        // String concatenation
        assert_eq!(
            eval_expr("\"hello\" ++ \" world\"", &row, None)
                .unwrap()
                .to_string(),
            "hello world"
        );

        // String comparison
        assert_eq!(
            eval_expr("\"a\" lt \"b\"", &row, None).unwrap().to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("\"hello\" eq \"hello\"", &row, None)
                .unwrap()
                .to_string(),
            "true"
        );
    }

    #[test]
    fn test_eval_list_operations() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        // List literal
        let result = eval_expr("[1, 2, 3]", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(1));
            }
            _ => panic!("Expected List"),
        }

        // List functions
        assert_eq!(
            eval_expr("len([1, 2, 3])", &row, None).unwrap().to_string(),
            "3"
        );
        assert_eq!(
            eval_expr("first([1, 2, 3])", &row, None)
                .unwrap()
                .to_string(),
            "1"
        );
        assert_eq!(
            eval_expr("last([1, 2, 3])", &row, None)
                .unwrap()
                .to_string(),
            "3"
        );
    }

    #[test]
    fn test_eval_lambda_operations() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        // map with lambda
        let result = eval_expr("map([1, 2, 3], x => x * 2)", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(2));
                assert_eq!(items[1], Value::Int(4));
                assert_eq!(items[2], Value::Int(6));
            }
            _ => panic!("Expected List"),
        }

        // filter with lambda
        let result = eval_expr("filter([1, 2, 3, 4], x => x > 2)", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], Value::Int(3));
                assert_eq!(items[1], Value::Int(4));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_variable_binding() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        let result = eval_expr("10 as @x; @x * 2", &row, None).unwrap();
        assert_eq!(result.to_string(), "20");

        let result = eval_expr("5 as @a; 3 as @b; @a + @b", &row, None).unwrap();
        assert_eq!(result.to_string(), "8");
    }

    #[test]
    fn test_eval_nested_function_calls() {
        use crate::libs::expr::eval_expr;

        let row = vec!["  hello  ".to_string()];

        assert_eq!(
            eval_expr("upper(trim(@1))", &row, None)
                .unwrap()
                .to_string(),
            "HELLO"
        );
    }

    #[test]
    fn test_eval_if_expression() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("if(true, 1, 0)", &row, None).unwrap().to_string(),
            "1"
        );
        assert_eq!(
            eval_expr("if(false, 1, 0)", &row, None)
                .unwrap()
                .to_string(),
            "0"
        );
        assert_eq!(
            eval_expr("if(5 > 3, \"yes\", \"no\")", &row, None)
                .unwrap()
                .to_string(),
            "yes"
        );
    }

    #[test]
    fn test_eval_type_checking_functions() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("is_int(42)", &row, None).unwrap().to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("is_float(3.14)", &row, None).unwrap().to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("is_string(\"hello\")", &row, None)
                .unwrap()
                .to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("is_bool(true)", &row, None).unwrap().to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("is_null(null)", &row, None).unwrap().to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("is_list([1, 2, 3])", &row, None)
                .unwrap()
                .to_string(),
            "true"
        );
    }

    #[test]
    fn test_eval_numeric_functions() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(eval_expr("abs(-5)", &row, None).unwrap().to_string(), "5");
        assert_eq!(
            eval_expr("round(3.7)", &row, None).unwrap().to_string(),
            "4"
        );
        assert_eq!(
            eval_expr("floor(3.7)", &row, None).unwrap().to_string(),
            "3"
        );
        assert_eq!(eval_expr("ceil(3.2)", &row, None).unwrap().to_string(), "4");
        assert_eq!(eval_expr("sqrt(16)", &row, None).unwrap().to_string(), "4");
        assert_eq!(
            eval_expr("max(1, 5, 3)", &row, None).unwrap().to_string(),
            "5"
        );
        assert_eq!(
            eval_expr("min(1, 5, 3)", &row, None).unwrap().to_string(),
            "1"
        );
    }

    #[test]
    fn test_eval_string_functions() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("upper(\"hello\")", &row, None)
                .unwrap()
                .to_string(),
            "HELLO"
        );
        assert_eq!(
            eval_expr("lower(\"WORLD\")", &row, None)
                .unwrap()
                .to_string(),
            "world"
        );
        assert_eq!(
            eval_expr("trim(\"  hello  \")", &row, None)
                .unwrap()
                .to_string(),
            "hello"
        );
        assert_eq!(
            eval_expr("len(\"hello\")", &row, None).unwrap().to_string(),
            "5"
        );
        assert_eq!(
            eval_expr("substr(\"hello\", 0, 3)", &row, None)
                .unwrap()
                .to_string(),
            "hel"
        );
    }

    #[test]
    fn test_eval_default_function() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("default(null, \"fallback\")", &row, None)
                .unwrap()
                .to_string(),
            "fallback"
        );
        assert_eq!(
            eval_expr("default(\"value\", \"fallback\")", &row, None)
                .unwrap()
                .to_string(),
            "value"
        );
    }

    #[test]
    fn test_eval_pipe_with_join() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        // join requires 2 args, must use _ explicitly
        assert_eq!(
            eval_expr("[1, 2, 3] | join(_, \",\")", &row, None)
                .unwrap()
                .to_string(),
            "1,2,3"
        );
    }

    #[test]
    fn test_eval_split_and_reverse() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        // split requires 2 args, must use _ explicitly
        let result =
            eval_expr("\"a,b,c\" | split(_, \",\") | reverse()", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::String("c".to_string()));
                assert_eq!(items[1], Value::String("b".to_string()));
                assert_eq!(items[2], Value::String("a".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_reduce() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        // Sum of [1, 2, 3, 4, 5] = 15
        let result = eval_expr(
            "reduce([1, 2, 3, 4, 5], 0, (acc, x) => acc + x)",
            &row,
            None,
        )
        .unwrap();
        assert_eq!(result.to_string(), "15");
    }

    #[test]
    fn test_eval_sort_by() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        let result = eval_expr(
            "sort_by([\"cherry\", \"apple\", \"pear\"], s => len(s))",
            &row,
            None,
        )
        .unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::String("pear".to_string()));
                assert_eq!(items[1], Value::String("apple".to_string()));
                assert_eq!(items[2], Value::String("cherry".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_take_while() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        let result =
            eval_expr("take_while([1, 2, 3, 4, 5], x => x < 4)", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(1));
                assert_eq!(items[1], Value::Int(2));
                assert_eq!(items[2], Value::Int(3));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_range() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        // range(upto)
        let result = eval_expr("range(5)", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 5);
                assert_eq!(items[0], Value::Int(0));
                assert_eq!(items[4], Value::Int(4));
            }
            _ => panic!("Expected List"),
        }

        // range(from, upto)
        let result = eval_expr("range(2, 5)", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(2));
                assert_eq!(items[2], Value::Int(4));
            }
            _ => panic!("Expected List"),
        }

        // range(from, upto, by)
        let result = eval_expr("range(0, 10, 3)", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 4);
                assert_eq!(items[0], Value::Int(0));
                assert_eq!(items[1], Value::Int(3));
                assert_eq!(items[2], Value::Int(6));
                assert_eq!(items[3], Value::Int(9));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_unique() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        let result = eval_expr("unique([1, 2, 2, 3, 3, 3])", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(1));
                assert_eq!(items[1], Value::Int(2));
                assert_eq!(items[2], Value::Int(3));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_sort() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        let result = eval_expr("sort([3, 1, 4, 1, 5])", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 5);
                assert_eq!(items[0], Value::Int(1));
                assert_eq!(items[1], Value::Int(1));
                assert_eq!(items[2], Value::Int(3));
                assert_eq!(items[3], Value::Int(4));
                assert_eq!(items[4], Value::Int(5));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_replace_nth() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        let result = eval_expr("replace_nth([1, 2, 3], 1, 99)", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(1));
                assert_eq!(items[1], Value::Int(99));
                assert_eq!(items[2], Value::Int(3));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_slice() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        let result = eval_expr("slice([1, 2, 3, 4, 5], 1, 4)", &row, None).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(2));
                assert_eq!(items[1], Value::Int(3));
                assert_eq!(items[2], Value::Int(4));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_eval_contains() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("contains(\"hello world\", \"world\")", &row, None)
                .unwrap()
                .to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("contains(\"hello world\", \"foo\")", &row, None)
                .unwrap()
                .to_string(),
            "false"
        );
    }

    #[test]
    fn test_eval_starts_with() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("starts_with(\"hello world\", \"hello\")", &row, None)
                .unwrap()
                .to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("starts_with(\"hello world\", \"world\")", &row, None)
                .unwrap()
                .to_string(),
            "false"
        );
    }

    #[test]
    fn test_eval_ends_with() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("ends_with(\"hello world\", \"world\")", &row, None)
                .unwrap()
                .to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("ends_with(\"hello world\", \"hello\")", &row, None)
                .unwrap()
                .to_string(),
            "false"
        );
    }

    #[test]
    fn test_eval_replace() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("replace(\"hello world\", \"world\", \"Rust\")", &row, None)
                .unwrap()
                .to_string(),
            "hello Rust"
        );
    }

    #[test]
    fn test_eval_truncate() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("truncate(\"hello world\", 8)", &row, None)
                .unwrap()
                .to_string(),
            "hello..."
        );
    }

    #[test]
    fn test_eval_wordcount() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("wordcount(\"hello world\")", &row, None)
                .unwrap()
                .to_string(),
            "2"
        );
        assert_eq!(
            eval_expr("wordcount(\"one two three four\")", &row, None)
                .unwrap()
                .to_string(),
            "4"
        );
    }

    #[test]
    fn test_eval_nth() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("nth([1, 2, 3], 0)", &row, None)
                .unwrap()
                .to_string(),
            "1"
        );
        assert_eq!(
            eval_expr("nth([1, 2, 3], 1)", &row, None)
                .unwrap()
                .to_string(),
            "2"
        );
        assert_eq!(
            eval_expr("nth([1, 2, 3], 2)", &row, None)
                .unwrap()
                .to_string(),
            "3"
        );
    }

    #[test]
    fn test_eval_type_function() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("type(42)", &row, None).unwrap().to_string(),
            "int"
        );
        assert_eq!(
            eval_expr("type(3.14)", &row, None).unwrap().to_string(),
            "float"
        );
        assert_eq!(
            eval_expr("type(\"hello\")", &row, None)
                .unwrap()
                .to_string(),
            "string"
        );
        assert_eq!(
            eval_expr("type(true)", &row, None).unwrap().to_string(),
            "bool"
        );
        assert_eq!(
            eval_expr("type(null)", &row, None).unwrap().to_string(),
            "null"
        );
        assert_eq!(
            eval_expr("type([1, 2, 3])", &row, None)
                .unwrap()
                .to_string(),
            "list"
        );
    }

    #[test]
    fn test_eval_is_numeric() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("is_numeric(42)", &row, None).unwrap().to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("is_numeric(3.14)", &row, None)
                .unwrap()
                .to_string(),
            "true"
        );
        assert_eq!(
            eval_expr("is_numeric(\"hello\")", &row, None)
                .unwrap()
                .to_string(),
            "false"
        );
    }

    #[test]
    fn test_eval_char_len() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        assert_eq!(
            eval_expr("char_len(\"hello\")", &row, None)
                .unwrap()
                .to_string(),
            "5"
        );
        // UTF-8 characters
        assert_eq!(
            eval_expr("char_len(\"你好\")", &row, None)
                .unwrap()
                .to_string(),
            "2"
        );
    }

    #[test]
    fn test_eval_reverse_string() {
        use crate::libs::expr::eval_expr;

        let row: Vec<String> = vec![];

        // split and join require 2 args, must use _ explicitly
        let result = eval_expr(
            "\"hello\" | split(_, \"\") | reverse() | join(_, \"\")",
            &row,
            None,
        )
        .unwrap();
        assert_eq!(result.to_string(), "olleh");
    }

    #[test]
    fn test_eval_complex_chain() {
        use crate::libs::expr::eval_expr;

        let row = vec!["1,2,3,4,5".to_string()];

        // Split, convert to int, double, filter > 4, join
        let result = eval_expr(
            "@1 | split(_,\",\") | map(_, x => int(x) * 2) | filter(_, x => x > 4) | join(_,\"-\")",
            &row,
            None,
        )
        .unwrap();
        assert_eq!(result.to_string(), "6-8-10");
    }

    #[test]
    fn test_eval_error_messages() {
        use crate::libs::expr::eval_expr;

        let row = vec!["10".to_string()];

        // Column index out of bounds
        let result = eval_expr("@2", &row, None);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("out of bounds"));

        // Unknown function
        let result = eval_expr("unknown_func()", &row, None);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Unknown function"));
    }

    #[test]
    fn test_global_var_basic() {
        let row: Vec<String> = vec![];
        let ctx = EvalContext::new(&row);

        // Initially unset global variable returns null
        assert_eq!(ctx.get_global("__test").unwrap(), Value::Null);

        // Set a global variable
        ctx.set_global("__test".to_string(), Value::Int(42));
        assert_eq!(ctx.get_global("__test").unwrap(), Value::Int(42));

        // Set another global variable
        ctx.set_global("__name".to_string(), Value::String("alice".to_string()));
        assert_eq!(
            ctx.get_global("__name").unwrap(),
            Value::String("alice".to_string())
        );
    }

    #[test]
    fn test_global_var_builtin() {
        let row: Vec<String> = vec![];
        let ctx = EvalContext::new(&row);

        // Set built-in globals
        ctx.set_builtin_globals(10, "test.tsv");

        assert_eq!(ctx.get_global("__index").unwrap(), Value::Int(10));
        assert_eq!(
            ctx.get_global("__file").unwrap(),
            Value::String("test.tsv".to_string())
        );
    }

    #[test]
    fn test_global_var_persistence() {
        use std::cell::RefCell;
        use std::rc::Rc;

        // Simulate two rows sharing the same globals
        let globals = Rc::new(RefCell::new(HashMap::new()));

        // First row processing
        let row1: Vec<String> = vec!["10".to_string()];
        let mut ctx1 = EvalContext::new(&row1);
        ctx1.globals = globals.clone();
        ctx1.set_global("__sum".to_string(), Value::Int(10));

        // Second row processing - shares the same globals
        let row2: Vec<String> = vec!["20".to_string()];
        let mut ctx2 = EvalContext::new(&row2);
        ctx2.globals = globals.clone();

        // Should see the value set by the first row
        assert_eq!(ctx2.get_global("__sum").unwrap(), Value::Int(10));

        // Update the value
        ctx2.set_global("__sum".to_string(), Value::Int(30));

        // First context should also see the updated value
        assert_eq!(ctx1.get_global("__sum").unwrap(), Value::Int(30));
    }

    #[test]
    fn test_eval_global_var_expr() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        // Set a global variable
        ctx.set_global("__counter".to_string(), Value::Int(5));

        // Evaluate global variable expression
        let expr = Expr::GlobalVar("__counter".to_string());
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_eval_global_var_bind() {
        let row = vec!["10".to_string()];
        let mut ctx = EvalContext::new(&row);

        // @1 + 5 as @__total
        let expr = Expr::Bind {
            expr: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
                right: Box::new(Expr::Int(5)),
            }),
            name: "__total".to_string(),
        };

        // Evaluate the bind expression
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Int(15));

        // Global variable should be accessible
        assert_eq!(ctx.get_global("__total").unwrap(), Value::Int(15));

        // Should also be accessible via GlobalVar expression
        let var_expr = Expr::GlobalVar("__total".to_string());
        assert_eq!(eval(&var_expr, &mut ctx).unwrap(), Value::Int(15));
    }

    #[test]
    fn test_eval_global_var_accumulator_pattern() {
        use std::cell::RefCell;
        use std::rc::Rc;

        // Simulate accumulator pattern across rows
        let globals = Rc::new(RefCell::new(HashMap::new()));

        // Row 1: value = 10
        let row1 = vec!["10".to_string()];
        let mut ctx1 = EvalContext::new(&row1);
        ctx1.globals = globals.clone();

        // default(@__sum, 0) + @1 as @__sum
        let current = ctx1.get_global("__sum").unwrap_or(Value::Null);
        let current = if current == Value::Null {
            Value::Int(0)
        } else {
            current
        };
        let sum = match current {
            Value::Int(n) => Value::Int(n + 10),
            _ => Value::Int(10),
        };
        ctx1.set_global("__sum".to_string(), sum);
        assert_eq!(ctx1.get_global("__sum").unwrap(), Value::Int(10));

        // Row 2: value = 20
        let row2 = vec!["20".to_string()];
        let mut ctx2 = EvalContext::new(&row2);
        ctx2.globals = globals.clone();

        let current = ctx2.get_global("__sum").unwrap_or(Value::Null);
        let current = if current == Value::Null {
            Value::Int(0)
        } else {
            current
        };
        let sum = match current {
            Value::Int(n) => Value::Int(n + 20),
            _ => Value::Int(20),
        };
        ctx2.set_global("__sum".to_string(), sum);
        assert_eq!(ctx2.get_global("__sum").unwrap(), Value::Int(30));

        // Row 3: value = 30
        let row3 = vec!["30".to_string()];
        let mut ctx3 = EvalContext::new(&row3);
        ctx3.globals = globals.clone();

        let current = ctx3.get_global("__sum").unwrap_or(Value::Null);
        let current = if current == Value::Null {
            Value::Int(0)
        } else {
            current
        };
        let sum = match current {
            Value::Int(n) => Value::Int(n + 30),
            _ => Value::Int(30),
        };
        ctx3.set_global("__sum".to_string(), sum);
        assert_eq!(ctx3.get_global("__sum").unwrap(), Value::Int(60));
    }

    #[test]
    fn test_set_builtin_globals() {
        let row = vec!["test".to_string()];
        let ctx = EvalContext::new(&row);
        ctx.set_builtin_globals(42, "file.tsv");

        // Verify __index
        assert_eq!(ctx.get_global("__index").unwrap(), Value::Int(42));
        // Verify __file
        assert_eq!(
            ctx.get_global("__file").unwrap(),
            Value::String("file.tsv".to_string())
        );
    }

    #[test]
    fn test_get_global_not_found_returns_null() {
        let row = vec!["test".to_string()];
        let ctx = EvalContext::new(&row);

        // Non-existent global should return Null, not error
        assert_eq!(ctx.get_global("__nonexistent").unwrap(), Value::Null);
    }

    #[test]
    fn test_fmt_with_column_ref_placeholder() {
        use crate::libs::expr::parser;

        let row = vec!["hello".to_string(), "world".to_string()];
        let headers = vec!["col1".to_string(), "col2".to_string()];
        let mut ctx = EvalContext::with_headers(&row, &headers);

        // Test fmt with %(@1) placeholder
        let expr = parser::parse("fmt('value: %(@1)', @1)").unwrap();
        let result = eval(&expr, &mut ctx).unwrap();
        assert_eq!(result, Value::String("value: hello".to_string()));
    }

    #[test]
    fn test_fmt_with_variable_placeholder() {
        use crate::libs::expr::functions::string::fmt_with_context;
        use ahash::HashMap;

        let row: Vec<String> = vec![];
        let ctx = EvalContext::new(&row);

        // Set up variables
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), Value::String("Alice".to_string()));

        // Test fmt with %(name) placeholder directly
        let args = vec![
            Value::String("Hello, %(name)!".to_string()),
            Value::String("placeholder".to_string()),
        ];
        let result = fmt_with_context(
            &args,
            Some(&row),
            Some(&variables),
            Some(&ctx.lambda_params),
            Some(ctx.globals.borrow()),
        )
        .unwrap();
        assert_eq!(result, Value::String("Hello, Alice!".to_string()));
    }

    #[test]
    fn test_fmt_with_lambda_param_placeholder() {
        use crate::libs::expr::functions::string::fmt_with_context;
        use ahash::HashMap;

        let row: Vec<String> = vec![];
        let ctx = EvalContext::new(&row);

        // Set up lambda params
        let mut lambda_params = HashMap::new();
        lambda_params.insert("x".to_string(), Value::Int(42));

        // Test fmt with %(x) placeholder
        let args = vec![
            Value::String("value: %(x)".to_string()),
            Value::String("placeholder".to_string()),
        ];
        let result = fmt_with_context(
            &args,
            Some(&row),
            Some(&ctx.variables),
            Some(&lambda_params),
            Some(ctx.globals.borrow()),
        )
        .unwrap();
        assert_eq!(result, Value::String("value: 42".to_string()));
    }

    #[test]
    fn test_fmt_with_global_placeholder() {
        use crate::libs::expr::functions::string::fmt_with_context;

        let row: Vec<String> = vec![];
        let ctx = EvalContext::new(&row);

        // Set up globals
        ctx.set_global("count".to_string(), Value::Int(100));

        // Test fmt with %(count) placeholder
        let args = vec![
            Value::String("count: %(count)".to_string()),
            Value::String("placeholder".to_string()),
        ];
        let result = fmt_with_context(
            &args,
            Some(&row),
            Some(&ctx.variables),
            Some(&ctx.lambda_params),
            Some(ctx.globals.borrow()),
        )
        .unwrap();
        assert_eq!(result, Value::String("count: 100".to_string()));
    }
}
