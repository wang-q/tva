/// Concrete expression evaluator
/// Optimized execution of ConcreteExpr with no string lookups
use crate::libs::expr::concrete::{ConcreteExpr, ConcretePipeRight};
use crate::libs::expr::parser::ast::{BinaryOp, UnaryOp};
use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use ahash::HashMapExt;

/// Evaluation context for concrete expressions
/// Uses Vec instead of HashMap for O(1) index-based access
pub struct ConcreteEvalContext<'a> {
    /// Row data as strings
    pub row: &'a [String],
    /// Variable bindings (index -> value)
    pub variables: Vec<Option<Value>>,
    /// Lambda parameter bindings (index -> value)
    pub lambda_params: Vec<Value>,
}

impl<'a> ConcreteEvalContext<'a> {
    pub fn new(row: &'a [String], var_count: usize) -> Self {
        Self {
            row,
            variables: vec![None; var_count],
            lambda_params: Vec::new(),
        }
    }

    pub fn with_lambda_params(
        row: &'a [String],
        var_count: usize,
        params: Vec<Value>,
    ) -> Self {
        Self {
            row,
            variables: vec![None; var_count],
            lambda_params: params,
        }
    }
}

/// Parse a string value into the appropriate Value type
/// Tries to parse as int, then float, then falls back to string
fn parse_value(s: &str) -> Value {
    if s.is_empty() {
        return Value::Null;
    }

    // Try integer first (faster than float parsing)
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

/// Evaluate a concrete expression
pub fn eval_concrete(
    expr: &ConcreteExpr,
    ctx: &mut ConcreteEvalContext,
) -> Result<Value, EvalError> {
    match expr {
        ConcreteExpr::Column(idx) => {
            if *idx < ctx.row.len() {
                Ok(parse_value(&ctx.row[*idx]))
            } else {
                Err(EvalError::ColumnIndexOutOfBounds(*idx))
            }
        }

        ConcreteExpr::Constant(v) => Ok(v.clone()),

        ConcreteExpr::Variable(idx) => ctx.variables[*idx]
            .clone()
            .ok_or_else(|| EvalError::VariableNotFound(format!("var_{}", idx))),

        ConcreteExpr::LambdaParam(idx) => {
            if *idx < ctx.lambda_params.len() {
                Ok(ctx.lambda_params[*idx].clone())
            } else {
                Err(EvalError::VariableNotFound(format!("param_{}", idx)))
            }
        }

        ConcreteExpr::Unary { op, expr } => {
            let val = eval_concrete(expr, ctx)?;
            eval_unary(*op, val)
        }

        ConcreteExpr::Binary { op, left, right } => {
            let left_val = eval_concrete(left, ctx)?;
            let right_val = eval_concrete(right, ctx)?;
            eval_binary(*op, left_val, right_val)
        }

        ConcreteExpr::Call { func, args } => {
            let arg_values: Result<Vec<_>, _> =
                args.iter().map(|arg| eval_concrete(arg, ctx)).collect();
            func(&arg_values?)
        }

        ConcreteExpr::MethodCall { object, func, args } => {
            let obj_val = eval_concrete(object, ctx)?;
            let arg_values: Result<Vec<_>, _> =
                args.iter().map(|arg| eval_concrete(arg, ctx)).collect();
            let mut all_args = vec![obj_val];
            all_args.extend(arg_values?);
            func(&all_args)
        }

        ConcreteExpr::Pipe { left, right } => {
            let left_val = eval_concrete(left, ctx)?;
            eval_pipe_right(right, left_val, ctx)
        }

        ConcreteExpr::Bind { expr, var_index } => {
            let val = eval_concrete(expr, ctx)?;
            if *var_index < ctx.variables.len() {
                ctx.variables[*var_index] = Some(val.clone());
            }
            Ok(val)
        }

        ConcreteExpr::Lambda { params, body: _ } => {
            // Return lambda as a value that can be called later
            // For now, we capture the current variable state
            Ok(Value::Lambda(
                crate::libs::expr::runtime::value::LambdaValue {
                    captured_vars: ahash::HashMap::new(), // TODO: capture variables
                    params: params.clone(),
                    body: crate::libs::expr::parser::ast::Expr::Null, // Placeholder - would need to store concrete expr
                },
            ))
        }

        ConcreteExpr::Block(exprs) => {
            let mut result = Value::Null;
            for expr in exprs {
                result = eval_concrete(expr, ctx)?;
            }
            Ok(result)
        }
    }
}

/// Evaluate pipe right side with the left value as input
fn eval_pipe_right(
    pipe_right: &ConcretePipeRight,
    left_val: Value,
    ctx: &mut ConcreteEvalContext,
) -> Result<Value, EvalError> {
    match pipe_right {
        ConcretePipeRight::Call { func, args } => {
            let mut arg_values = vec![left_val];
            for arg in args {
                arg_values.push(eval_concrete(arg, ctx)?);
            }
            func(&arg_values)
        }

        ConcretePipeRight::CallWithPlaceholder {
            func,
            placeholder_index,
            args,
        } => {
            let mut arg_values = Vec::new();
            let mut arg_idx = 0;
            for i in 0..=args.len() {
                if i == *placeholder_index {
                    arg_values.push(left_val.clone());
                } else if arg_idx < args.len() {
                    arg_values.push(eval_concrete(&args[arg_idx], ctx)?);
                    arg_idx += 1;
                }
            }
            func(&arg_values)
        }
    }
}

/// Evaluate unary operation
fn eval_unary(op: UnaryOp, val: Value) -> Result<Value, EvalError> {
    match op {
        UnaryOp::Neg => match val {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(EvalError::TypeError(format!(
                "Cannot negate {}",
                val.type_name()
            ))),
        },
        UnaryOp::Not => match val {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(EvalError::TypeError(format!(
                "Cannot negate {}",
                val.type_name()
            ))),
        },
    }
}

/// Evaluate binary operation
fn eval_binary(op: BinaryOp, left: Value, right: Value) -> Result<Value, EvalError> {
    use BinaryOp::*;

    match op {
        Add => (left + right)
            .ok_or_else(|| EvalError::TypeError("Add requires numbers".to_string())),
        Sub => (left - right)
            .ok_or_else(|| EvalError::TypeError("Sub requires numbers".to_string())),
        Mul => (left * right)
            .ok_or_else(|| EvalError::TypeError("Mul requires numbers".to_string())),
        Div => {
            // Check for division by zero
            match &right {
                Value::Int(0) => Err(EvalError::DivisionByZero),
                Value::Float(f) if *f == 0.0 => Err(EvalError::DivisionByZero),
                _ => (left / right).ok_or_else(|| {
                    EvalError::TypeError("Div requires numbers".to_string())
                }),
            }
        }
        Mod => left.modulo(&right),
        Pow => left
            .pow(&right)
            .ok_or_else(|| EvalError::TypeError("Pow requires numbers".to_string())),
        Eq => Ok(Value::Bool(left == right)),
        Ne => Ok(Value::Bool(left != right)),
        Lt => left.lt(&right).ok_or_else(|| {
            EvalError::TypeError("Comparison requires numbers".to_string())
        }),
        Le => left.le(&right).ok_or_else(|| {
            EvalError::TypeError("Comparison requires numbers".to_string())
        }),
        Gt => left.gt(&right).ok_or_else(|| {
            EvalError::TypeError("Comparison requires numbers".to_string())
        }),
        Ge => left.ge(&right).ok_or_else(|| {
            EvalError::TypeError("Comparison requires numbers".to_string())
        }),
        // String comparisons
        BinaryOp::StrEq => Ok(Value::Bool(left.to_string() == right.to_string())),
        BinaryOp::StrNe => Ok(Value::Bool(left.to_string() != right.to_string())),
        BinaryOp::StrLt => Ok(Value::Bool(left.to_string() < right.to_string())),
        BinaryOp::StrLe => Ok(Value::Bool(left.to_string() <= right.to_string())),
        BinaryOp::StrGt => Ok(Value::Bool(left.to_string() > right.to_string())),
        BinaryOp::StrGe => Ok(Value::Bool(left.to_string() >= right.to_string())),
        And => match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l && r)),
            _ => Err(EvalError::TypeError("And requires booleans".to_string())),
        },
        Or => match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l || r)),
            _ => Err(EvalError::TypeError("Or requires booleans".to_string())),
        },
        Concat => left.concat(&right),
    }
}
