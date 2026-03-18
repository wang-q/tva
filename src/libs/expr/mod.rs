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
/// Returns error for bare identifiers (ColumnRef::Bare) that are not lambda params
pub fn resolve_columns(expr: &mut Expr, headers: &[String]) -> Result<(), ExprError> {
    use parser::ast::ColumnRef;

    match expr {
        Expr::ColumnRef(ColumnRef::Bare(name)) => {
            // Bare identifiers are not allowed - user should use @ prefix
            return Err(ExprError::Eval(EvalError::ColumnNotFound(format!(
                "Bare identifier '{}' is not allowed; use '@{}' for column references",
                name, name
            ))));
        }
        Expr::ColumnRef(ColumnRef::Name(name)) => {
            // Find index (1-based) for the column name
            if let Some(idx) = headers.iter().position(|h| h == name) {
                *expr = Expr::ColumnRef(ColumnRef::Index(idx + 1));
            }
            // If not found, keep as Name (will error at runtime)
        }
        Expr::Unary { expr: inner, .. } => {
            resolve_columns(inner, headers)?;
        }
        Expr::Binary { left, right, .. } => {
            resolve_columns(left, headers)?;
            resolve_columns(right, headers)?;
        }
        Expr::Call { args, .. } => {
            for arg in args {
                resolve_columns(arg, headers)?;
            }
        }
        Expr::MethodCall { object, args, .. } => {
            resolve_columns(object, headers)?;
            for arg in args {
                resolve_columns(arg, headers)?;
            }
        }
        Expr::Pipe { left, right } => {
            resolve_columns(left, headers)?;
            resolve_pipe_right(right, headers)?;
        }
        Expr::Bind { expr: inner, .. } => {
            resolve_columns(inner, headers)?;
        }
        Expr::Block(exprs) => {
            for e in exprs {
                resolve_columns(e, headers)?;
            }
        }
        Expr::List(items) => {
            for item in items {
                resolve_columns(item, headers)?;
            }
        }
        Expr::Lambda { body, .. } => {
            resolve_columns(body, headers)?;
        }
        // ColumnRef::Index, Variable, LambdaParam, literals - no resolution needed
        _ => {}
    }
    Ok(())
}

/// Resolve columns in pipe right-hand side
fn resolve_pipe_right(
    expr: &mut parser::ast::PipeRight,
    headers: &[String],
) -> Result<(), ExprError> {
    use parser::ast::PipeRight;

    match expr {
        PipeRight::Call { args, .. } => {
            for arg in args {
                resolve_columns(arg, headers)?;
            }
        }
        PipeRight::CallWithPlaceholder { args, .. } => {
            for arg in args {
                resolve_columns(arg, headers)?;
            }
        }
    }
    Ok(())
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
    resolve_columns(&mut expr, headers)?;

    // Evaluate with resolved indices
    let mut ctx = runtime::EvalContext::with_headers(row, headers);
    Ok(runtime::eval(&expr, &mut ctx)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::ast::{BinaryOp, ColumnRef, Expr, UnaryOp};

    #[test]
    fn test_parse_cached_basic() {
        // Clear cache first
        clear_cache();
        assert_eq!(cache_size(), 0);

        // Parse and cache
        let expr1 = parse_cached("@1 + @2").unwrap();
        assert_eq!(cache_size(), 1);

        // Parse same expression - should hit cache
        let expr2 = parse_cached("@1 + @2").unwrap();
        assert_eq!(cache_size(), 1); // Still 1, not 2

        // Parse different expression
        let _expr3 = parse_cached("@1 * @2").unwrap();
        assert_eq!(cache_size(), 2);

        // Verify expressions are equivalent
        match (&expr1, &expr2) {
            (Expr::Binary { op: op1, .. }, Expr::Binary { op: op2, .. }) => {
                assert_eq!(*op1, *op2);
            }
            _ => panic!("Expected Binary expressions"),
        }
    }

    #[test]
    fn test_clear_cache() {
        parse_cached("@1 + 1").unwrap();
        parse_cached("@1 + 2").unwrap();
        assert_eq!(cache_size(), 2);

        clear_cache();
        assert_eq!(cache_size(), 0);
    }

    #[test]
    fn test_cache_size_empty() {
        clear_cache();
        assert_eq!(cache_size(), 0);
    }

    #[test]
    fn test_resolve_columns_by_name() {
        let mut expr = Expr::ColumnRef(ColumnRef::Name("price".to_string()));
        let headers = vec!["name".to_string(), "price".to_string(), "qty".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        // Should be resolved to index 2 (1-based)
        match expr {
            Expr::ColumnRef(ColumnRef::Index(idx)) => {
                assert_eq!(idx, 2);
            }
            _ => panic!("Expected ColumnRef::Index, got {:?}", expr),
        }
    }

    #[test]
    fn test_resolve_columns_name_not_found() {
        // Column name not in headers - should remain as Name
        let mut expr = Expr::ColumnRef(ColumnRef::Name("unknown".to_string()));
        let headers = vec!["name".to_string(), "price".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::ColumnRef(ColumnRef::Name(name)) => {
                assert_eq!(name, "unknown");
            }
            _ => panic!("Expected ColumnRef::Name to remain unchanged"),
        }
    }

    #[test]
    fn test_resolve_columns_index_unchanged() {
        // Index references should remain unchanged
        let mut expr = Expr::ColumnRef(ColumnRef::Index(1));
        let headers = vec!["name".to_string(), "price".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::ColumnRef(ColumnRef::Index(idx)) => {
                assert_eq!(idx, 1);
            }
            _ => panic!("Expected ColumnRef::Index to remain unchanged"),
        }
    }

    #[test]
    fn test_resolve_columns_in_binary() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::ColumnRef(ColumnRef::Name("price".to_string()))),
            right: Box::new(Expr::ColumnRef(ColumnRef::Name("qty".to_string()))),
        };
        let headers = vec!["name".to_string(), "price".to_string(), "qty".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::Binary { left, right, .. } => {
                match (*left, *right) {
                    (
                        Expr::ColumnRef(ColumnRef::Index(left_idx)),
                        Expr::ColumnRef(ColumnRef::Index(right_idx)),
                    ) => {
                        assert_eq!(left_idx, 2); // price is at index 2
                        assert_eq!(right_idx, 3); // qty is at index 3
                    }
                    _ => panic!("Expected both columns to be resolved to indices"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_resolve_columns_in_unary() {
        let mut expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::ColumnRef(ColumnRef::Name("value".to_string()))),
        };
        let headers = vec!["value".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::Unary { expr, .. } => match *expr {
                Expr::ColumnRef(ColumnRef::Index(idx)) => {
                    assert_eq!(idx, 1);
                }
                _ => panic!("Expected inner column to be resolved"),
            },
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_resolve_columns_in_call() {
        let mut expr = Expr::Call {
            name: "max".to_string(),
            args: vec![
                Expr::ColumnRef(ColumnRef::Name("a".to_string())),
                Expr::ColumnRef(ColumnRef::Name("b".to_string())),
            ],
        };
        let headers = vec!["a".to_string(), "b".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::Call { args, .. } => {
                assert_eq!(args.len(), 2);
                match (&args[0], &args[1]) {
                    (
                        Expr::ColumnRef(ColumnRef::Index(1)),
                        Expr::ColumnRef(ColumnRef::Index(2)),
                    ) => {}
                    _ => panic!("Expected args to be resolved to indices 1 and 2"),
                }
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_resolve_columns_in_method_call() {
        let mut expr = Expr::MethodCall {
            object: Box::new(Expr::ColumnRef(ColumnRef::Name("name".to_string()))),
            name: "upper".to_string(),
            args: vec![],
        };
        let headers = vec!["name".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::MethodCall { object, .. } => match *object {
                Expr::ColumnRef(ColumnRef::Index(1)) => {}
                _ => panic!("Expected object to be resolved to index 1"),
            },
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_resolve_columns_in_list() {
        let mut expr = Expr::List(vec![
            Expr::ColumnRef(ColumnRef::Name("a".to_string())),
            Expr::ColumnRef(ColumnRef::Name("b".to_string())),
        ]);
        let headers = vec!["a".to_string(), "b".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::List(items) => {
                assert_eq!(items.len(), 2);
                match (&items[0], &items[1]) {
                    (
                        Expr::ColumnRef(ColumnRef::Index(1)),
                        Expr::ColumnRef(ColumnRef::Index(2)),
                    ) => {}
                    _ => panic!("Expected list items to be resolved"),
                }
            }
            _ => panic!("Expected List expression"),
        }
    }

    #[test]
    fn test_resolve_columns_in_lambda() {
        let mut expr = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::LambdaParam("x".to_string())),
                right: Box::new(Expr::ColumnRef(ColumnRef::Name("offset".to_string()))),
            }),
        };
        let headers = vec!["offset".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::Lambda { body, .. } => match *body {
                Expr::Binary { right, .. } => match *right {
                    Expr::ColumnRef(ColumnRef::Index(1)) => {}
                    _ => panic!("Expected column in lambda body to be resolved"),
                },
                _ => panic!("Expected Binary in lambda body"),
            },
            _ => panic!("Expected Lambda expression"),
        }
    }

    #[test]
    fn test_resolve_columns_in_block() {
        let mut expr = Expr::Block(vec![
            Expr::ColumnRef(ColumnRef::Name("a".to_string())),
            Expr::ColumnRef(ColumnRef::Name("b".to_string())),
        ]);
        let headers = vec!["a".to_string(), "b".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::Block(exprs) => {
                assert_eq!(exprs.len(), 2);
                match (&exprs[0], &exprs[1]) {
                    (
                        Expr::ColumnRef(ColumnRef::Index(1)),
                        Expr::ColumnRef(ColumnRef::Index(2)),
                    ) => {}
                    _ => panic!("Expected block expressions to be resolved"),
                }
            }
            _ => panic!("Expected Block expression"),
        }
    }

    #[test]
    fn test_resolve_columns_in_bind() {
        let mut expr = Expr::Bind {
            name: "total".to_string(),
            expr: Box::new(Expr::ColumnRef(ColumnRef::Name("price".to_string()))),
        };
        let headers = vec!["price".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::Bind { expr, .. } => match *expr {
                Expr::ColumnRef(ColumnRef::Index(1)) => {}
                _ => panic!("Expected bound expression to be resolved"),
            },
            _ => panic!("Expected Bind expression"),
        }
    }

    #[test]
    fn test_fold_constants_unary_neg_int() {
        let mut expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::Int(42)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Int(-42)));
    }

    #[test]
    fn test_fold_constants_unary_neg_float() {
        let mut expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::Float(3.14)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f + 3.14).abs() < 0.001),
            _ => panic!("Expected Float(-3.14)"),
        }
    }

    #[test]
    fn test_fold_constants_unary_not() {
        let mut expr = Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expr::Bool(true)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(false)));
    }

    #[test]
    fn test_fold_constants_binary_add_int() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Int(5)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Int(15)));
    }

    #[test]
    fn test_fold_constants_binary_sub_int() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Int(3)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Int(7)));
    }

    #[test]
    fn test_fold_constants_binary_mul_int() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Int(6)),
            right: Box::new(Expr::Int(7)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Int(42)));
    }

    #[test]
    fn test_fold_constants_binary_div_int() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Int(20)),
            right: Box::new(Expr::Int(4)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Int(5)));
    }

    #[test]
    fn test_fold_constants_binary_div_by_zero() {
        // Division by zero should not fold
        let mut expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Int(0)),
        };
        fold_constants(&mut expr);
        // Should remain unchanged
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Div));
                assert!(matches!(*left, Expr::Int(10)));
                assert!(matches!(*right, Expr::Int(0)));
            }
            _ => panic!("Expected expression to remain unchanged"),
        }
    }

    #[test]
    fn test_fold_constants_binary_mod() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Mod,
            left: Box::new(Expr::Int(17)),
            right: Box::new(Expr::Int(5)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Int(2)));
    }

    #[test]
    fn test_fold_constants_binary_pow_int() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Pow,
            left: Box::new(Expr::Int(2)),
            right: Box::new(Expr::Int(10)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Int(1024)));
    }

    #[test]
    fn test_fold_constants_binary_pow_large_exponent() {
        // Large exponents should not fold
        let mut expr = Expr::Binary {
            op: BinaryOp::Pow,
            left: Box::new(Expr::Int(2)),
            right: Box::new(Expr::Int(200)),
        };
        fold_constants(&mut expr);
        // Should remain unchanged
        match expr {
            Expr::Binary { op, .. } => {
                assert!(matches!(op, BinaryOp::Pow));
            }
            _ => panic!("Expected expression to remain unchanged for large exponent"),
        }
    }

    #[test]
    fn test_fold_constants_binary_float_arithmetic() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Float(1.5)),
            right: Box::new(Expr::Float(2.5)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 4.0).abs() < 0.001),
            _ => panic!("Expected Float(4.0)"),
        }
    }

    #[test]
    fn test_fold_constants_binary_mixed_int_float() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Float(2.5)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 12.5).abs() < 0.001),
            _ => panic!("Expected Float(12.5)"),
        }
    }

    #[test]
    fn test_fold_constants_string_concat() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Concat,
            left: Box::new(Expr::String("hello".to_string())),
            right: Box::new(Expr::String(" world".to_string())),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::String(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected String(\"hello world\")"),
        }
    }

    #[test]
    fn test_fold_constants_comparison_int() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::Int(5)),
            right: Box::new(Expr::Int(10)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_comparison_float() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Ge,
            left: Box::new(Expr::Float(3.14)),
            right: Box::new(Expr::Float(2.71)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_string_comparison() {
        let mut expr = Expr::Binary {
            op: BinaryOp::StrEq,
            left: Box::new(Expr::String("test".to_string())),
            right: Box::new(Expr::String("test".to_string())),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_binary_float_sub() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::Float(5.0)),
            right: Box::new(Expr::Float(3.0)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 2.0).abs() < 0.001),
            _ => panic!("Expected Float(2.0)"),
        }
    }

    #[test]
    fn test_fold_constants_binary_float_mul() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Float(2.5)),
            right: Box::new(Expr::Float(4.0)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 10.0).abs() < 0.001),
            _ => panic!("Expected Float(10.0)"),
        }
    }

    #[test]
    fn test_fold_constants_binary_float_div() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Float(10.0)),
            right: Box::new(Expr::Float(2.0)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 5.0).abs() < 0.001),
            _ => panic!("Expected Float(5.0)"),
        }
    }

    #[test]
    fn test_fold_constants_binary_float_pow() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Pow,
            left: Box::new(Expr::Float(2.0)),
            right: Box::new(Expr::Float(3.0)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 8.0).abs() < 0.001),
            _ => panic!("Expected Float(8.0)"),
        }
    }

    #[test]
    fn test_fold_constants_binary_mixed_sub_int_float() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Float(3.5)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 6.5).abs() < 0.001),
            _ => panic!("Expected Float(6.5)"),
        }
    }

    #[test]
    fn test_fold_constants_binary_mixed_sub_float_int() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::Float(10.5)),
            right: Box::new(Expr::Int(3)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 7.5).abs() < 0.001),
            _ => panic!("Expected Float(7.5)"),
        }
    }

    #[test]
    fn test_fold_constants_binary_mixed_mul_int_float() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Int(5)),
            right: Box::new(Expr::Float(2.5)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 12.5).abs() < 0.001),
            _ => panic!("Expected Float(12.5)"),
        }
    }

    #[test]
    fn test_fold_constants_binary_mixed_mul_float_int() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Float(2.5)),
            right: Box::new(Expr::Int(4)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 10.0).abs() < 0.001),
            _ => panic!("Expected Float(10.0)"),
        }
    }

    #[test]
    fn test_fold_constants_binary_mixed_div_int_float() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::Float(2.5)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 4.0).abs() < 0.001),
            _ => panic!("Expected Float(4.0)"),
        }
    }

    #[test]
    fn test_fold_constants_binary_mixed_div_float_int() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Float(10.0)),
            right: Box::new(Expr::Int(4)),
        };
        fold_constants(&mut expr);
        match expr {
            Expr::Float(f) => assert!((f - 2.5).abs() < 0.001),
            _ => panic!("Expected Float(2.5)"),
        }
    }

    #[test]
    fn test_fold_constants_comparison_int_eq() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::Int(5)),
            right: Box::new(Expr::Int(5)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_comparison_int_ne() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expr::Int(5)),
            right: Box::new(Expr::Int(10)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_comparison_float_eq() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::Float(3.14)),
            right: Box::new(Expr::Float(3.14)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_comparison_float_ne() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expr::Float(3.14)),
            right: Box::new(Expr::Float(2.71)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_comparison_float_lt() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::Float(2.71)),
            right: Box::new(Expr::Float(3.14)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_comparison_float_le() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Le,
            left: Box::new(Expr::Float(3.14)),
            right: Box::new(Expr::Float(3.14)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_comparison_float_gt() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Float(3.14)),
            right: Box::new(Expr::Float(2.71)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_string_comparison_ne() {
        let mut expr = Expr::Binary {
            op: BinaryOp::StrNe,
            left: Box::new(Expr::String("a".to_string())),
            right: Box::new(Expr::String("b".to_string())),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_string_comparison_lt() {
        let mut expr = Expr::Binary {
            op: BinaryOp::StrLt,
            left: Box::new(Expr::String("a".to_string())),
            right: Box::new(Expr::String("b".to_string())),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_string_comparison_le() {
        let mut expr = Expr::Binary {
            op: BinaryOp::StrLe,
            left: Box::new(Expr::String("a".to_string())),
            right: Box::new(Expr::String("a".to_string())),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_string_comparison_gt() {
        let mut expr = Expr::Binary {
            op: BinaryOp::StrGt,
            left: Box::new(Expr::String("b".to_string())),
            right: Box::new(Expr::String("a".to_string())),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_string_comparison_ge() {
        let mut expr = Expr::Binary {
            op: BinaryOp::StrGe,
            left: Box::new(Expr::String("b".to_string())),
            right: Box::new(Expr::String("a".to_string())),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_logical_and() {
        let mut expr = Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Bool(true)),
            right: Box::new(Expr::Bool(false)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(false)));
    }

    #[test]
    fn test_fold_constants_logical_or() {
        let mut expr = Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Bool(false)),
            right: Box::new(Expr::Bool(true)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Bool(true)));
    }

    #[test]
    fn test_fold_constants_nested() {
        // (2 + 3) * 4 should fold to 20
        let mut expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Int(2)),
                right: Box::new(Expr::Int(3)),
            }),
            right: Box::new(Expr::Int(4)),
        };
        fold_constants(&mut expr);
        assert!(matches!(expr, Expr::Int(20)));
    }

    #[test]
    fn test_fold_constants_in_list() {
        let mut expr = Expr::List(vec![
            Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Int(1)),
                right: Box::new(Expr::Int(2)),
            },
            Expr::Binary {
                op: BinaryOp::Mul,
                left: Box::new(Expr::Int(3)),
                right: Box::new(Expr::Int(4)),
            },
        ]);
        fold_constants(&mut expr);

        match expr {
            Expr::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0], Expr::Int(3)));
                assert!(matches!(items[1], Expr::Int(12)));
            }
            _ => panic!("Expected List expression"),
        }
    }

    #[test]
    fn test_fold_constants_in_call() {
        let mut expr = Expr::Call {
            name: "max".to_string(),
            args: vec![
                Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Int(1)),
                    right: Box::new(Expr::Int(2)),
                },
                Expr::Int(5),
            ],
        };
        fold_constants(&mut expr);

        match expr {
            Expr::Call { args, .. } => {
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0], Expr::Int(3)));
                assert!(matches!(args[1], Expr::Int(5)));
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_fold_constants_no_fold_variable() {
        // Variables should not be folded
        let mut expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::ColumnRef(ColumnRef::Index(1))),
            right: Box::new(Expr::Int(5)),
        };
        fold_constants(&mut expr);

        // Should remain unchanged
        match expr {
            Expr::Binary { left, right, .. } => {
                assert!(matches!(*left, Expr::ColumnRef(ColumnRef::Index(1))));
                assert!(matches!(*right, Expr::Int(5)));
            }
            _ => panic!("Expected expression to remain unchanged"),
        }
    }

    #[test]
    fn test_eval_expr_basic() {
        let row = vec!["10".to_string(), "20".to_string()];
        let result = eval_expr("@1 + @2", &row, None).unwrap();
        match result {
            runtime::value::Value::Int(n) => assert_eq!(n, 30),
            _ => panic!("Expected Int(30)"),
        }
    }

    #[test]
    fn test_eval_expr_with_headers() {
        let row = vec!["100".to_string(), "200".to_string()];
        let headers = vec!["price".to_string(), "qty".to_string()];
        let result = eval_expr("@price + @qty", &row, Some(&headers)).unwrap();
        match result {
            runtime::value::Value::Int(n) => assert_eq!(n, 300),
            _ => panic!("Expected Int(300)"),
        }
    }

    #[test]
    fn test_eval_expr_parse_error() {
        let row: Vec<String> = vec![];
        let result = eval_expr("@", &row, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_expr_cached_basic() {
        clear_cache();
        let row = vec!["5".to_string(), "3".to_string()];

        // First call - parses and caches
        let result1 = eval_expr_cached("@1 * @2", &row, None).unwrap();

        // Second call - uses cache
        let result2 = eval_expr_cached("@1 * @2", &row, None).unwrap();

        match (result1, result2) {
            (runtime::value::Value::Int(n1), runtime::value::Value::Int(n2)) => {
                assert_eq!(n1, 15);
                assert_eq!(n2, 15);
            }
            _ => panic!("Expected Int(15)"),
        }

        // Cache should contain at least this expression (may contain more from other tests)
        assert!(cache_size() >= 1);
    }

    #[test]
    fn test_eval_expr_cached_resolved() {
        clear_cache();
        let row = vec!["10".to_string(), "5".to_string()];
        let headers = vec!["a".to_string(), "b".to_string()];

        let result = eval_expr_cached_resolved("@a + @b", &row, &headers).unwrap();

        match result {
            runtime::value::Value::Int(n) => assert_eq!(n, 15),
            _ => panic!("Expected Int(15)"),
        }
    }

    #[test]
    fn test_resolve_columns_in_pipe_call() {
        use parser::ast::PipeRight;

        let mut expr = Expr::Pipe {
            left: Box::new(Expr::ColumnRef(ColumnRef::Name("value".to_string()))),
            right: Box::new(PipeRight::Call {
                name: "abs".to_string(),
                args: vec![],
            }),
        };
        let headers = vec!["value".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::Pipe { left, right } => {
                match *left {
                    Expr::ColumnRef(ColumnRef::Index(1)) => {}
                    _ => panic!("Expected left to be resolved to index 1"),
                }
                match *right {
                    PipeRight::Call { name, args } => {
                        assert_eq!(name, "abs");
                        assert!(args.is_empty());
                    }
                    _ => panic!("Expected PipeRight::Call"),
                }
            }
            _ => panic!("Expected Pipe expression"),
        }
    }

    #[test]
    fn test_resolve_columns_in_pipe_with_placeholder() {
        use parser::ast::PipeRight;

        let mut expr = Expr::Pipe {
            left: Box::new(Expr::ColumnRef(ColumnRef::Name("desc".to_string()))),
            right: Box::new(PipeRight::CallWithPlaceholder {
                name: "substr".to_string(),
                args: vec![Expr::Int(0), Expr::Int(50)],
            }),
        };
        let headers = vec!["name".to_string(), "desc".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::Pipe { left, right } => {
                match *left {
                    Expr::ColumnRef(ColumnRef::Index(2)) => {}
                    _ => panic!("Expected left to be resolved to index 2"),
                }
                match *right {
                    PipeRight::CallWithPlaceholder { name, args } => {
                        assert_eq!(name, "substr");
                        assert_eq!(args.len(), 2);
                    }
                    _ => panic!("Expected PipeRight::CallWithPlaceholder"),
                }
            }
            _ => panic!("Expected Pipe expression"),
        }
    }

    #[test]
    fn test_resolve_columns_in_pipe_nested() {
        use parser::ast::PipeRight;

        let mut expr = Expr::Pipe {
            left: Box::new(Expr::ColumnRef(ColumnRef::Name("name".to_string()))),
            right: Box::new(PipeRight::CallWithPlaceholder {
                name: "replace".to_string(),
                args: vec![
                    Expr::ColumnRef(ColumnRef::Name("old".to_string())),
                    Expr::ColumnRef(ColumnRef::Name("new".to_string())),
                ],
            }),
        };
        let headers = vec!["name".to_string(), "old".to_string(), "new".to_string()];

        resolve_columns(&mut expr, &headers).unwrap();

        match expr {
            Expr::Pipe { left, right } => {
                match *left {
                    Expr::ColumnRef(ColumnRef::Index(1)) => {}
                    _ => panic!("Expected left to be resolved to index 1"),
                }
                match *right {
                    PipeRight::CallWithPlaceholder { args, .. } => {
                        assert_eq!(args.len(), 2);
                        match (&args[0], &args[1]) {
                            (
                                Expr::ColumnRef(ColumnRef::Index(2)),
                                Expr::ColumnRef(ColumnRef::Index(3)),
                            ) => {}
                            _ => {
                                panic!("Expected args to be resolved to indices 2 and 3")
                            }
                        }
                    }
                    _ => panic!("Expected PipeRight::CallWithPlaceholder"),
                }
            }
            _ => panic!("Expected Pipe expression"),
        }
    }

    #[test]
    fn test_fold_constants_in_pipe_call() {
        use parser::ast::PipeRight;

        let mut expr = Expr::Pipe {
            left: Box::new(Expr::Int(5)),
            right: Box::new(PipeRight::Call {
                name: "abs".to_string(),
                args: vec![],
            }),
        };
        fold_constants(&mut expr);

        match expr {
            Expr::Pipe { left, right } => {
                assert!(matches!(*left, Expr::Int(5)));
                match *right {
                    PipeRight::Call { name, args } => {
                        assert_eq!(name, "abs");
                        assert!(args.is_empty());
                    }
                    _ => panic!("Expected PipeRight::Call"),
                }
            }
            _ => panic!("Expected Pipe expression"),
        }
    }

    #[test]
    fn test_fold_constants_in_pipe_with_placeholder() {
        use parser::ast::PipeRight;

        let mut expr = Expr::Pipe {
            left: Box::new(Expr::Int(10)),
            right: Box::new(PipeRight::CallWithPlaceholder {
                name: "pow".to_string(),
                args: vec![Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Int(1)),
                    right: Box::new(Expr::Int(2)),
                }],
            }),
        };
        fold_constants(&mut expr);

        match expr {
            Expr::Pipe { left, right } => {
                assert!(matches!(*left, Expr::Int(10)));
                match *right {
                    PipeRight::CallWithPlaceholder { name, args } => {
                        assert_eq!(name, "pow");
                        assert_eq!(args.len(), 1);
                        // The arg should be folded from (1 + 2) to 3
                        assert!(matches!(args[0], Expr::Int(3)));
                    }
                    _ => panic!("Expected PipeRight::CallWithPlaceholder"),
                }
            }
            _ => panic!("Expected Pipe expression"),
        }
    }

    #[test]
    fn test_fold_constants_in_pipe_call_with_args() {
        use parser::ast::PipeRight;

        // PipeRight::Call with constant arguments that should be folded
        let mut expr = Expr::Pipe {
            left: Box::new(Expr::Int(10)),
            right: Box::new(PipeRight::Call {
                name: "max".to_string(),
                args: vec![
                    Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Int(1)),
                        right: Box::new(Expr::Int(2)),
                    },
                    Expr::Int(5),
                ],
            }),
        };
        fold_constants(&mut expr);

        match expr {
            Expr::Pipe { left, right } => {
                assert!(matches!(*left, Expr::Int(10)));
                match *right {
                    PipeRight::Call { name, args } => {
                        assert_eq!(name, "max");
                        assert_eq!(args.len(), 2);
                        // First arg should be folded from (1 + 2) to 3
                        assert!(matches!(args[0], Expr::Int(3)));
                        assert!(matches!(args[1], Expr::Int(5)));
                    }
                    _ => panic!("Expected PipeRight::Call"),
                }
            }
            _ => panic!("Expected Pipe expression"),
        }
    }

    #[test]
    fn test_fold_constants_in_method_call() {
        // Method call with constant expression in object and args
        let mut expr = Expr::MethodCall {
            object: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Int(1)),
                right: Box::new(Expr::Int(2)),
            }),
            name: "to_string".to_string(),
            args: vec![],
        };
        fold_constants(&mut expr);

        match expr {
            Expr::MethodCall { object, name, args } => {
                assert_eq!(name, "to_string");
                assert!(args.is_empty());
                // Object should be folded from (1 + 2) to 3
                assert!(matches!(*object, Expr::Int(3)));
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_fold_constants_in_method_call_with_args() {
        // Method call with constant arguments
        let mut expr = Expr::MethodCall {
            object: Box::new(Expr::String("hello".to_string())),
            name: "replace".to_string(),
            args: vec![
                Expr::String("l".to_string()),
                Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Int(1)),
                    right: Box::new(Expr::Int(2)),
                },
            ],
        };
        fold_constants(&mut expr);

        match expr {
            Expr::MethodCall { object, args, .. } => {
                assert!(matches!(*object, Expr::String(_)));
                assert_eq!(args.len(), 2);
                // Second arg should be folded from (1 + 2) to 3
                assert!(matches!(args[1], Expr::Int(3)));
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_fold_constants_in_lambda() {
        // Lambda with constant expression in body
        let mut expr = Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::LambdaParam("x".to_string())),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::Mul,
                    left: Box::new(Expr::Int(2)),
                    right: Box::new(Expr::Int(3)),
                }),
            }),
        };
        fold_constants(&mut expr);

        match expr {
            Expr::Lambda { params, body } => {
                assert_eq!(params.len(), 1);
                match *body {
                    Expr::Binary { left, right, .. } => {
                        assert!(matches!(*left, Expr::LambdaParam(_)));
                        // Right side should be folded from (2 * 3) to 6
                        assert!(matches!(*right, Expr::Int(6)));
                    }
                    _ => panic!("Expected Binary in lambda body"),
                }
            }
            _ => panic!("Expected Lambda expression"),
        }
    }

    #[test]
    fn test_fold_constants_no_fold_method_call() {
        // Method call on non-constant should not be folded
        let mut expr = Expr::MethodCall {
            object: Box::new(Expr::ColumnRef(ColumnRef::Name("value".to_string()))),
            name: "to_string".to_string(),
            args: vec![],
        };
        fold_constants(&mut expr);

        // Should remain unchanged
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert_eq!(name, "to_string");
                assert!(args.is_empty());
                assert!(
                    matches!(*object, Expr::ColumnRef(ColumnRef::Name(s)) if s == "value")
                );
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_resolve_columns_bare_identifier_rejected() {
        // Bare identifiers should be rejected during column resolution
        let mut expr = Expr::ColumnRef(ColumnRef::Bare("price".to_string()));
        let headers = vec!["name".to_string(), "price".to_string()];

        let result = resolve_columns(&mut expr, &headers);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Bare identifier 'price' is not allowed"));
        assert!(err_msg.contains("use '@price'"));
    }

    #[test]
    fn test_resolve_columns_bare_in_binary_rejected() {
        // Bare identifiers in binary expressions should be rejected
        let mut expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::ColumnRef(ColumnRef::Bare("a".to_string()))),
            right: Box::new(Expr::Int(1)),
        };
        let headers = vec!["a".to_string(), "b".to_string()];

        let result = resolve_columns(&mut expr, &headers);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Bare identifier 'a' is not allowed"));
    }

    #[test]
    fn test_resolve_columns_bare_in_call_rejected() {
        // Bare identifiers in function calls should be rejected
        let mut expr = Expr::Call {
            name: "fmt".to_string(),
            args: vec![
                Expr::String("%()".to_string()),
                Expr::ColumnRef(ColumnRef::Bare("value".to_string())),
            ],
        };
        let headers = vec!["value".to_string()];

        let result = resolve_columns(&mut expr, &headers);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Bare identifier 'value' is not allowed"));
    }

    #[test]
    fn test_parse_bare_identifier_becomes_bare() {
        // Parsing a bare identifier should create ColumnRef::Bare
        let expr = parser::parse("price").unwrap();
        match expr {
            Expr::ColumnRef(ColumnRef::Bare(name)) => {
                assert_eq!(name, "price");
            }
            _ => panic!("Expected ColumnRef::Bare, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_at_identifier_becomes_name() {
        // Parsing @identifier should create ColumnRef::Name
        let expr = parser::parse("@price").unwrap();
        match expr {
            Expr::ColumnRef(ColumnRef::Name(name)) => {
                assert_eq!(name, "price");
            }
            _ => panic!("Expected ColumnRef::Name, got {:?}", expr),
        }
    }

    #[test]
    fn test_lambda_param_not_bare() {
        // Lambda parameters should be converted to LambdaParam, not Bare
        let expr = parser::parse("x => x + 1").unwrap();
        match expr {
            Expr::Lambda { params, body } => {
                assert_eq!(params, vec!["x"]);
                match *body {
                    Expr::Binary { left, .. } => match *left {
                        Expr::LambdaParam(name) => assert_eq!(name, "x"),
                        _ => panic!("Expected LambdaParam, got {:?}", left),
                    },
                    _ => panic!("Expected Binary in lambda body"),
                }
            }
            _ => panic!("Expected Lambda expression"),
        }
    }
}
