pub mod value;

use crate::libs::expr::parser::ast::{BinaryOp, ColumnRef, Expr, PipeRight, UnaryOp};
use std::collections::HashMap;
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
}

impl<'a> EvalContext<'a> {
    pub fn new(row: &'a [String]) -> Self {
        Self {
            row,
            headers: None,
            variables: HashMap::new(),
            lambda_params: HashMap::new(),
        }
    }

    pub fn with_headers(row: &'a [String], headers: &'a [String]) -> Self {
        Self {
            row,
            headers: Some(headers),
            variables: HashMap::new(),
            lambda_params: HashMap::new(),
        }
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
        },
        Expr::Variable(name) => ctx.get_variable(name),
        Expr::LambdaParam(name) => ctx.get_lambda_param(name),
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
                BinaryOp::Div => (left_val / right_val).ok_or(EvalError::DivisionByZero),
                BinaryOp::Mod => (left_val % right_val).ok_or(EvalError::DivisionByZero),
                BinaryOp::Pow => left_val
                    .pow(&right_val)
                    .ok_or(EvalError::TypeError("expected numeric".to_string())),
                // String concatenation
                BinaryOp::Concat => {
                    Ok(Value::String(left_val.as_string() + &right_val.as_string()))
                }
                // Comparison (numeric)
                BinaryOp::Eq => Ok(left_val.eq(&right_val)),
                BinaryOp::Ne => Ok(left_val.ne(&right_val)),
                BinaryOp::Lt => left_val
                    .lt(&right_val)
                    .ok_or(EvalError::TypeError("expected comparable".to_string())),
                BinaryOp::Le => left_val
                    .le(&right_val)
                    .ok_or(EvalError::TypeError("expected comparable".to_string())),
                BinaryOp::Gt => left_val
                    .gt(&right_val)
                    .ok_or(EvalError::TypeError("expected comparable".to_string())),
                BinaryOp::Ge => left_val
                    .ge(&right_val)
                    .ok_or(EvalError::TypeError("expected comparable".to_string())),
                // Comparison (string)
                BinaryOp::StrEq => {
                    Ok(Value::Bool(left_val.as_string() == right_val.as_string()))
                }
                BinaryOp::StrNe => {
                    Ok(Value::Bool(left_val.as_string() != right_val.as_string()))
                }
                BinaryOp::StrLt => {
                    Ok(Value::Bool(left_val.as_string() < right_val.as_string()))
                }
                BinaryOp::StrLe => {
                    Ok(Value::Bool(left_val.as_string() <= right_val.as_string()))
                }
                BinaryOp::StrGt => {
                    Ok(Value::Bool(left_val.as_string() > right_val.as_string()))
                }
                BinaryOp::StrGe => {
                    Ok(Value::Bool(left_val.as_string() >= right_val.as_string()))
                }
                // Logical
                BinaryOp::And => {
                    Ok(Value::Bool(left_val.as_bool() && right_val.as_bool()))
                }
                BinaryOp::Or => {
                    Ok(Value::Bool(left_val.as_bool() || right_val.as_bool()))
                }
            }
        }
        Expr::Call { name, args } => {
            let arg_values: Vec<Value> = args
                .iter()
                .map(|arg| eval(arg, ctx))
                .collect::<Result<Vec<_>, _>>()?;
            let registry = crate::libs::expr::functions::FunctionRegistry::new();
            registry.call(name, &arg_values)
        }
        Expr::MethodCall { object, name, args } => {
            // Evaluate the object first
            let obj_val = eval(object, ctx)?;
            // Build args: object as first arg, followed by method args
            let mut arg_values = vec![obj_val];
            for arg in args {
                arg_values.push(eval(arg, ctx)?);
            }
            let registry = crate::libs::expr::functions::FunctionRegistry::new();
            registry.call(name, &arg_values)
        }
        Expr::Pipe { left, right } => {
            let left_val = eval(left, ctx)?;
            eval_pipe_right(right, left_val, ctx)
        }
        Expr::Bind { expr, name } => {
            let val = eval(expr, ctx)?;
            ctx.set_variable(name.clone(), val.clone());
            Ok(val)
        }
        Expr::Block(exprs) => {
            let mut result = Value::Null;
            for expr in exprs {
                result = eval(expr, ctx)?;
            }
            Ok(result)
        }
        Expr::Lambda { params, body } => Ok(Value::Lambda(value::LambdaValue {
            params: params.clone(),
            body: *body.clone(),
        })),
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
            // Build args: piped value as first arg, followed by explicit args
            let mut arg_values = vec![piped_value];
            for arg in args {
                arg_values.push(eval(arg, ctx)?);
            }
            let registry = crate::libs::expr::functions::FunctionRegistry::new();
            registry.call(name, &arg_values)
        }
        PipeRight::CallWithPlaceholder { name, args } => {
            // Replace placeholder _ with piped value
            let mut arg_values = Vec::new();
            // First arg is always the piped value in placeholder mode
            arg_values.push(piped_value);
            // Add remaining args
            for arg in args {
                arg_values.push(eval(arg, ctx)?);
            }
            let registry = crate::libs::expr::functions::FunctionRegistry::new();
            registry.call(name, &arg_values)
        }
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
}
