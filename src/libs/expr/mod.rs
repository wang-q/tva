//! Expression engine for TVA
//!
//! Provides parsing and evaluation of expressions like `@1 + @2 * 3`

pub mod functions;
pub mod parser;
pub mod runtime;

#[cfg(test)]
mod tests;

use parser::ParseError;
use runtime::EvalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExprError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("Evaluation error: {0}")]
    Eval(#[from] EvalError),
}

/// Parse and evaluate an expression in one step
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
