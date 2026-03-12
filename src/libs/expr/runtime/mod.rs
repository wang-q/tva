pub mod value;

use crate::libs::expr::parser::ast::{BinaryOp, ColumnRef, Expr, UnaryOp};
use thiserror::Error;
use value::Value;

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Column index {0} out of bounds")]
    ColumnIndexOutOfBounds(usize),
    #[error("Column '{0}' not found")]
    ColumnNotFound(String),
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
}

impl<'a> EvalContext<'a> {
    pub fn new(row: &'a [String]) -> Self {
        Self { row, headers: None }
    }

    pub fn with_headers(row: &'a [String], headers: &'a [String]) -> Self {
        Self {
            row,
            headers: Some(headers),
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
pub fn eval(expr: &Expr, ctx: &EvalContext) -> Result<Value, EvalError> {
    match expr {
        Expr::ColumnRef(col_ref) => match col_ref {
            ColumnRef::Index(idx) => ctx.get_by_index(*idx),
            ColumnRef::Name(name) => ctx.get_by_name(name),
        },
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Float(n) => Ok(Value::Float(*n)),
        Expr::String(s) => Ok(Value::String(s.clone())),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Null => Ok(Value::Null),
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
                // Comparison
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::expr::parser::ast::{ColumnRef, Expr};

    #[test]
    fn test_eval_column_ref() {
        let row = vec!["10".to_string(), "20".to_string(), "hello".to_string()];
        let ctx = EvalContext::new(&row);

        let expr = Expr::ColumnRef(ColumnRef::Index(1));
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Int(10));

        let expr = Expr::ColumnRef(ColumnRef::Index(2));
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Int(20));

        let expr = Expr::ColumnRef(ColumnRef::Index(3));
        assert_eq!(
            eval(&expr, &ctx).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_eval_arithmetic() {
        let row = vec!["10".to_string(), "3".to_string()];
        let ctx = EvalContext::new(&row);

        // @1 + @2
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
            right: Box::new(Expr::ColumnRef(ColumnRef::Index(2))),
        };
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Int(13));

        // @1 * @2
        let expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
            right: Box::new(Expr::ColumnRef(ColumnRef::Index(2))),
        };
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_eval_number_literal() {
        let row: Vec<String> = vec![];
        let ctx = EvalContext::new(&row);

        // 42
        let expr = Expr::Int(42);
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Int(42));

        // 3.14
        let expr = Expr::Float(3.14);
        match eval(&expr, &ctx).unwrap() {
            Value::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_eval_comparison() {
        let row = vec!["10".to_string(), "20".to_string()];
        let ctx = EvalContext::new(&row);

        // @1 < @2
        let expr = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
            right: Box::new(Expr::ColumnRef(ColumnRef::Index(2))),
        };
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Bool(true));

        // @1 == @2
        let expr = Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
            right: Box::new(Expr::ColumnRef(ColumnRef::Index(2))),
        };
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_logical() {
        let row: Vec<String> = vec![];
        let ctx = EvalContext::new(&row);

        // true && false
        let expr = Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Bool(true)),
            right: Box::new(Expr::Bool(false)),
        };
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Bool(false));

        // true || false
        let expr = Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Bool(true)),
            right: Box::new(Expr::Bool(false)),
        };
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_unary() {
        let row = vec!["5".to_string()];
        let ctx = EvalContext::new(&row);

        // -@1
        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
        };
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Int(-5));

        // !true
        let expr = Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expr::Bool(true)),
        };
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_with_headers() {
        let row = vec!["Alice".to_string(), "30".to_string()];
        let headers = vec!["name".to_string(), "age".to_string()];
        let ctx = EvalContext::with_headers(&row, &headers);

        let expr = Expr::ColumnRef(ColumnRef::Name("name".to_string()));
        assert_eq!(
            eval(&expr, &ctx).unwrap(),
            Value::String("Alice".to_string())
        );

        let expr = Expr::ColumnRef(ColumnRef::Name("age".to_string()));
        assert_eq!(eval(&expr, &ctx).unwrap(), Value::Int(30));
    }
}
