//! Expression engine for TVA
//!
//! Provides parsing and evaluation of expressions like `@1 + @2 * 3`

pub mod functions;
pub mod parser;
pub mod runtime;

use ahash::{HashMap, HashMapExt};
use parser::{ast::Expr, ParseError};
use runtime::EvalError;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExprError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("Evaluation error: {0}")]
    Eval(#[from] EvalError),
}

/// Cache for parsed expressions to avoid re-parsing
/// Uses a simple HashMap with Mutex for thread safety
static EXPR_CACHE: Mutex<Option<HashMap<String, Expr>>> = Mutex::new(None);

/// Parse expression with caching
/// Repeated parsing of the same expression string will return the cached AST
pub fn parse_cached(expr_str: &str) -> Result<Expr, ParseError> {
    // Fast path: check cache without locking if possible
    // (In practice, we need to lock for thread safety)

    let mut cache_guard = EXPR_CACHE.lock().unwrap();

    // Initialize cache if empty
    if cache_guard.is_none() {
        *cache_guard = Some(HashMap::new());
    }

    let cache = cache_guard.as_mut().unwrap();

    // Check if already cached
    if let Some(expr) = cache.get(expr_str) {
        return Ok(expr.clone());
    }

    // Parse and cache
    let expr = parser::parse(expr_str)?;
    cache.insert(expr_str.to_string(), expr.clone());
    Ok(expr)
}

/// Clear the expression cache
pub fn clear_cache() {
    let mut cache_guard = EXPR_CACHE.lock().unwrap();
    *cache_guard = None;
}

/// Get cache size (for debugging/monitoring)
pub fn cache_size() -> usize {
    let cache_guard = EXPR_CACHE.lock().unwrap();
    cache_guard.as_ref().map(|c| c.len()).unwrap_or(0)
}

/// Resolve column names to indices in an expression
/// This transforms @name -> @index for faster runtime access
pub fn resolve_columns(expr: &mut Expr, headers: &[String]) {
    use parser::ast::ColumnRef;

    match expr {
        Expr::ColumnRef(ColumnRef::Name(name)) => {
            // Find index (1-based) for the column name
            if let Some(idx) = headers.iter().position(|h| h == name) {
                *expr = Expr::ColumnRef(ColumnRef::Index(idx + 1));
            }
            // If not found, keep as Name (will error at runtime)
        }
        Expr::Unary { expr: inner, .. } => {
            resolve_columns(inner, headers);
        }
        Expr::Binary { left, right, .. } => {
            resolve_columns(left, headers);
            resolve_columns(right, headers);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                resolve_columns(arg, headers);
            }
        }
        Expr::MethodCall { object, args, .. } => {
            resolve_columns(object, headers);
            for arg in args {
                resolve_columns(arg, headers);
            }
        }
        Expr::Pipe { left, right } => {
            resolve_columns(left, headers);
            resolve_pipe_right(right, headers);
        }
        Expr::Bind { expr: inner, .. } => {
            resolve_columns(inner, headers);
        }
        Expr::Block(exprs) => {
            for e in exprs {
                resolve_columns(e, headers);
            }
        }
        Expr::List(items) => {
            for item in items {
                resolve_columns(item, headers);
            }
        }
        Expr::Lambda { body, .. } => {
            resolve_columns(body, headers);
        }
        // ColumnRef::Index, Variable, LambdaParam, literals - no resolution needed
        _ => {}
    }
}

/// Resolve columns in pipe right-hand side
fn resolve_pipe_right(expr: &mut parser::ast::PipeRight, headers: &[String]) {
    use parser::ast::PipeRight;

    match expr {
        PipeRight::Call { args, .. } => {
            for arg in args {
                resolve_columns(arg, headers);
            }
        }
        PipeRight::CallWithPlaceholder { args, .. } => {
            for arg in args {
                resolve_columns(arg, headers);
            }
        }
    }
}

/// Fold constant expressions
/// Evaluates constant sub-expressions at compile time
/// Examples:
/// - `2 + 3` -> `5`
/// - `"hello" ++ " world"` -> `"hello world"`
/// - `!true` -> `false`
pub fn fold_constants(expr: &mut Expr) {
    // First, recursively fold children
    match expr {
        Expr::Unary { expr: inner, .. } => {
            fold_constants(inner);
        }
        Expr::Binary { left, right, .. } => {
            fold_constants(left);
            fold_constants(right);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                fold_constants(arg);
            }
        }
        Expr::MethodCall { object, args, .. } => {
            fold_constants(object);
            for arg in args {
                fold_constants(arg);
            }
        }
        Expr::Pipe { left, right } => {
            fold_constants(left);
            fold_pipe_right(right);
        }
        Expr::Bind { expr: inner, .. } => {
            fold_constants(inner);
        }
        Expr::Block(exprs) => {
            for e in exprs {
                fold_constants(e);
            }
        }
        Expr::List(items) => {
            for item in items {
                fold_constants(item);
            }
        }
        Expr::Lambda { body, .. } => {
            fold_constants(body);
        }
        _ => {}
    }

    // Then, try to fold this expression if it's a constant
    match expr {
        // Unary operations on constants
        Expr::Unary { op, expr: inner } => {
            if let Some(val) = try_fold_unary(*op, inner) {
                *expr = val;
            }
        }
        // Binary operations on constants
        Expr::Binary { op, left, right } => {
            if let Some(val) = try_fold_binary(*op, left, right) {
                *expr = val;
            }
        }
        _ => {}
    }
}

/// Fold constants in pipe right-hand side
fn fold_pipe_right(expr: &mut parser::ast::PipeRight) {
    use parser::ast::PipeRight;

    match expr {
        PipeRight::Call { args, .. } => {
            for arg in args {
                fold_constants(arg);
            }
        }
        PipeRight::CallWithPlaceholder { args, .. } => {
            for arg in args {
                fold_constants(arg);
            }
        }
    }
}

/// Try to fold a unary operation
fn try_fold_unary(op: parser::ast::UnaryOp, expr: &Expr) -> Option<Expr> {
    use parser::ast::UnaryOp;

    match (op, expr) {
        (UnaryOp::Neg, Expr::Int(n)) => Some(Expr::Int(-n)),
        (UnaryOp::Neg, Expr::Float(f)) => Some(Expr::Float(-f)),
        (UnaryOp::Not, Expr::Bool(b)) => Some(Expr::Bool(!b)),
        _ => None,
    }
}

/// Try to fold a binary operation
fn try_fold_binary(
    op: parser::ast::BinaryOp,
    left: &Expr,
    right: &Expr,
) -> Option<Expr> {
    use parser::ast::BinaryOp;

    match (op, left, right) {
        // Integer arithmetic
        (BinaryOp::Add, Expr::Int(a), Expr::Int(b)) => Some(Expr::Int(a + b)),
        (BinaryOp::Sub, Expr::Int(a), Expr::Int(b)) => Some(Expr::Int(a - b)),
        (BinaryOp::Mul, Expr::Int(a), Expr::Int(b)) => Some(Expr::Int(a * b)),
        (BinaryOp::Div, Expr::Int(a), Expr::Int(b)) => {
            if *b != 0 {
                Some(Expr::Int(a / b))
            } else {
                None
            }
        }
        (BinaryOp::Mod, Expr::Int(a), Expr::Int(b)) => {
            if *b != 0 {
                Some(Expr::Int(a % b))
            } else {
                None
            }
        }
        (BinaryOp::Pow, Expr::Int(a), Expr::Int(b)) => {
            if *b >= 0 && *b <= 100 {
                // Limit exponent to avoid huge numbers
                Some(Expr::Int(a.pow(*b as u32)))
            } else {
                None
            }
        }
        // Float arithmetic
        (BinaryOp::Add, Expr::Float(a), Expr::Float(b)) => Some(Expr::Float(a + b)),
        (BinaryOp::Sub, Expr::Float(a), Expr::Float(b)) => Some(Expr::Float(a - b)),
        (BinaryOp::Mul, Expr::Float(a), Expr::Float(b)) => Some(Expr::Float(a * b)),
        (BinaryOp::Div, Expr::Float(a), Expr::Float(b)) => Some(Expr::Float(a / b)),
        (BinaryOp::Pow, Expr::Float(a), Expr::Float(b)) => Some(Expr::Float(a.powf(*b))),
        // Mixed int/float
        (BinaryOp::Add, Expr::Int(a), Expr::Float(b)) => {
            Some(Expr::Float(*a as f64 + b))
        }
        (BinaryOp::Add, Expr::Float(a), Expr::Int(b)) => {
            Some(Expr::Float(a + *b as f64))
        }
        (BinaryOp::Sub, Expr::Int(a), Expr::Float(b)) => {
            Some(Expr::Float(*a as f64 - b))
        }
        (BinaryOp::Sub, Expr::Float(a), Expr::Int(b)) => {
            Some(Expr::Float(a - *b as f64))
        }
        (BinaryOp::Mul, Expr::Int(a), Expr::Float(b)) => {
            Some(Expr::Float(*a as f64 * b))
        }
        (BinaryOp::Mul, Expr::Float(a), Expr::Int(b)) => {
            Some(Expr::Float(a * *b as f64))
        }
        (BinaryOp::Div, Expr::Int(a), Expr::Float(b)) => {
            Some(Expr::Float(*a as f64 / b))
        }
        (BinaryOp::Div, Expr::Float(a), Expr::Int(b)) => {
            Some(Expr::Float(a / *b as f64))
        }
        // String concatenation
        (BinaryOp::Concat, Expr::String(a), Expr::String(b)) => {
            Some(Expr::String(format!("{}{}", a, b)))
        }
        // Comparison (numeric)
        (BinaryOp::Eq, Expr::Int(a), Expr::Int(b)) => Some(Expr::Bool(a == b)),
        (BinaryOp::Ne, Expr::Int(a), Expr::Int(b)) => Some(Expr::Bool(a != b)),
        (BinaryOp::Lt, Expr::Int(a), Expr::Int(b)) => Some(Expr::Bool(a < b)),
        (BinaryOp::Le, Expr::Int(a), Expr::Int(b)) => Some(Expr::Bool(a <= b)),
        (BinaryOp::Gt, Expr::Int(a), Expr::Int(b)) => Some(Expr::Bool(a > b)),
        (BinaryOp::Ge, Expr::Int(a), Expr::Int(b)) => Some(Expr::Bool(a >= b)),
        // Comparison (float)
        (BinaryOp::Eq, Expr::Float(a), Expr::Float(b)) => Some(Expr::Bool(a == b)),
        (BinaryOp::Ne, Expr::Float(a), Expr::Float(b)) => Some(Expr::Bool(a != b)),
        (BinaryOp::Lt, Expr::Float(a), Expr::Float(b)) => Some(Expr::Bool(a < b)),
        (BinaryOp::Le, Expr::Float(a), Expr::Float(b)) => Some(Expr::Bool(a <= b)),
        (BinaryOp::Gt, Expr::Float(a), Expr::Float(b)) => Some(Expr::Bool(a > b)),
        (BinaryOp::Ge, Expr::Float(a), Expr::Float(b)) => Some(Expr::Bool(a >= b)),
        // Comparison (string)
        (BinaryOp::StrEq, Expr::String(a), Expr::String(b)) => Some(Expr::Bool(a == b)),
        (BinaryOp::StrNe, Expr::String(a), Expr::String(b)) => Some(Expr::Bool(a != b)),
        (BinaryOp::StrLt, Expr::String(a), Expr::String(b)) => Some(Expr::Bool(a < b)),
        (BinaryOp::StrLe, Expr::String(a), Expr::String(b)) => Some(Expr::Bool(a <= b)),
        (BinaryOp::StrGt, Expr::String(a), Expr::String(b)) => Some(Expr::Bool(a > b)),
        (BinaryOp::StrGe, Expr::String(a), Expr::String(b)) => Some(Expr::Bool(a >= b)),
        // Logical
        (BinaryOp::And, Expr::Bool(a), Expr::Bool(b)) => Some(Expr::Bool(*a && *b)),
        (BinaryOp::Or, Expr::Bool(a), Expr::Bool(b)) => Some(Expr::Bool(*a || *b)),
        _ => None,
    }
}

/// Parse and evaluate an expression in one step (without caching)
pub fn eval_expr(
    expr_str: impl AsRef<str>,
    row: &[String],
    headers: Option<&[String]>,
) -> Result<runtime::value::Value, ExprError> {
    let expr = parser::parse(expr_str.as_ref())?;
    let mut ctx = match headers {
        Some(h) => runtime::EvalContext::with_headers(row, h),
        None => runtime::EvalContext::new(row),
    };
    Ok(runtime::eval(&expr, &mut ctx)?)
}

/// Parse and evaluate an expression with caching
/// Use this for repeated evaluations of the same expression
pub fn eval_expr_cached(
    expr_str: impl AsRef<str>,
    row: &[String],
    headers: Option<&[String]>,
) -> Result<runtime::value::Value, ExprError> {
    let expr = parse_cached(expr_str.as_ref())?;
    let mut ctx = match headers {
        Some(h) => runtime::EvalContext::with_headers(row, h),
        None => runtime::EvalContext::new(row),
    };
    Ok(runtime::eval(&expr, &mut ctx)?)
}

/// Parse, resolve columns, and evaluate with caching
/// This is the most optimized path for repeated evaluations with the same headers
pub fn eval_expr_cached_resolved(
    expr_str: impl AsRef<str>,
    row: &[String],
    headers: &[String],
) -> Result<runtime::value::Value, ExprError> {
    // Get cached AST
    let mut expr = parse_cached(expr_str.as_ref())?;

    // Resolve column names to indices (modifies the cloned AST)
    resolve_columns(&mut expr, headers);

    // Evaluate with resolved indices
    let mut ctx = runtime::EvalContext::with_headers(row, headers);
    Ok(runtime::eval(&expr, &mut ctx)?)
}
