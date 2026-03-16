pub mod ast;

mod builder;

use ast::Expr;
use builder::*;
use pest::Parser;
use pest_derive::Parser as PestParser;
use thiserror::Error;

#[derive(PestParser)]
#[grammar = "libs/expr/parser/grammar.pest"]
struct ExprParser;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
    #[error("Invalid column index: {0}")]
    InvalidColumnIndex(String),
    #[error("Unexpected rule: {0:?}")]
    UnexpectedRule(Rule),
    #[error("Empty expression")]
    EmptyExpression,
}

/// Parse an expression string into an AST
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let pairs = ExprParser::parse(Rule::full_expr, input)?;
    for pair in pairs {
        match pair.as_rule() {
            // full_expr is silent (_{...}), so we get expr_list directly
            Rule::expr_list => {
                return build_full_expr(pair);
            }
            Rule::full_expr => {
                return build_full_expr(pair);
            }
            _ => {}
        }
    }
    Err(ParseError::EmptyExpression)
}
