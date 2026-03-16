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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::expr::parser::ast::ColumnRef;

    #[test]
    fn test_parse_simple_int() {
        let result = parse("42");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::Int(42)));
    }

    #[test]
    fn test_parse_simple_float() {
        let result = parse("3.14");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::Float(f) if (f - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_parse_simple_string() {
        let result = parse("\"hello\"");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::String(s) if s == "hello"));
    }

    #[test]
    fn test_parse_simple_bool() {
        let result = parse("true");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::Bool(true)));

        let result = parse("false");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::Bool(false)));
    }

    #[test]
    fn test_parse_null() {
        let result = parse("null");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::Null));
    }

    #[test]
    fn test_parse_column_ref_by_index() {
        let result = parse("@1");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::ColumnRef(ColumnRef::Index(idx)) => {
                assert_eq!(idx, 1);
            }
            _ => panic!("Expected ColumnRef::Index"),
        }
    }

    #[test]
    fn test_parse_column_ref_by_name() {
        let result = parse("@name");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::ColumnRef(ColumnRef::Name(name)) => {
                assert_eq!(name, "name");
            }
            _ => panic!("Expected ColumnRef::Name"),
        }
    }

    #[test]
    fn test_parse_arithmetic() {
        let result = parse("1 + 2 * 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function_call() {
        let result = parse("abs(-5)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lambda() {
        let result = parse("map([1,2,3], x => x * 2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_input() {
        let result = parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let result = parse("1 + + 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_complex_expression() {
        let expr = r#"
            @price * @qty as @total;
            @total * 1.1 as @with_tax;
            @with_tax
        "#;
        let result = parse(expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_list() {
        let result = parse("[1, 2, 3]");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::List(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_pipe() {
        let result = parse("[1,2,3] | len()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_variable_bind() {
        let result = parse("1 + 2 as @result");
        assert!(result.is_ok());
    }
}
