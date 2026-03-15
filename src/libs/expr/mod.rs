//! Expression engine for TVA
//!
//! Provides parsing and evaluation of expressions like `@1 + @2 * 3`

pub mod functions;
pub mod parser;
pub mod runtime;

use parser::{ast::Expr, ParseError};
use runtime::EvalError;
use std::collections::HashMap;
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
