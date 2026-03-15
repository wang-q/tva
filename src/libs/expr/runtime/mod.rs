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
    fn test_parse_value_empty() {
        // Empty string should parse to Null
        let row = vec!["".to_string()];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::ColumnRef(ColumnRef::Index(1));
        assert_eq!(eval(&expr, &mut ctx).unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_value_float() {
        // Float string should parse to Float
        let row = vec!["3.14".to_string()];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::ColumnRef(ColumnRef::Index(1));
        match eval(&expr, &mut ctx).unwrap() {
            Value::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_parse_value_string() {
        // Non-numeric string should remain as String
        let row = vec!["hello".to_string()];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::ColumnRef(ColumnRef::Index(1));
        assert_eq!(
            eval(&expr, &mut ctx).unwrap(),
            Value::String("hello".to_string())
        );
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

        // Test @text |> replace("world", "Rust")
        let expr = Expr::Pipe {
            left: Box::new(Expr::ColumnRef(ColumnRef::Name("text".to_string()))),
            right: Box::new(PipeRight::CallWithPlaceholder {
                name: "replace".to_string(),
                args: vec![
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
    fn test_eval_unknown_function() {
        let row: Vec<String> = vec![];
        let mut ctx = EvalContext::new(&row);

        let expr = Expr::Call {
            name: "unknown_function_xyz".to_string(),
            args: vec![],
        };
        let result = eval(&expr, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown function"));
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
}
