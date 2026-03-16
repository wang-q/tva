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
    use crate::libs::expr::parser::ast::{ColumnRef, UnaryOp};

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

    #[test]
    fn test_parse_single_quoted_string() {
        let result = parse("'hello world'");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::String(s) if s == "hello world"));
    }

    #[test]
    fn test_parse_string_with_escapes() {
        let result = parse("\"hello\\nworld\"");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::String(s) if s == "hello\nworld"));
    }

    #[test]
    fn test_parse_string_with_tab() {
        let result = parse("\"hello\\tworld\"");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::String(s) if s == "hello\tworld"));
    }

    #[test]
    fn test_parse_negative_number() {
        let result = parse("-42");
        assert!(result.is_ok());
        assert!(
            matches!(result.unwrap(), Expr::Unary { op: UnaryOp::Neg, expr } if matches!(&*expr, Expr::Int(42)))
        );
    }

    #[test]
    fn test_parse_negative_float() {
        let result = parse("-3.14");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_scientific_notation() {
        let result = parse("1e10");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::Float(f) if (f - 1e10).abs() < 0.1));

        let result = parse("2.5e-3");
        assert!(result.is_ok());
        assert!(
            matches!(result.unwrap(), Expr::Float(f) if (f - 0.0025).abs() < 0.0001)
        );
    }

    #[test]
    fn test_parse_column_ref_whole_row() {
        let result = parse("@0");
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            Expr::ColumnRef(ColumnRef::WholeRow)
        ));
    }

    #[test]
    fn test_parse_column_ref_quoted_name() {
        let result = parse("@\"user name\"");
        assert!(result.is_ok());
        assert!(
            matches!(result.unwrap(), Expr::ColumnRef(ColumnRef::Name(name)) if name == "user name")
        );

        let result = parse("@'user name'");
        assert!(result.is_ok());
        assert!(
            matches!(result.unwrap(), Expr::ColumnRef(ColumnRef::Name(name)) if name == "user name")
        );
    }

    #[test]
    fn test_parse_pipe_expression() {
        let result = parse("\"hello\" | upper()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_pipe_with_placeholder() {
        let result = parse("\"hello world\" | substr(_, 0, 5)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_method_call() {
        let result = parse("@name.trim()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_method_call_chain() {
        let result = parse("@name.trim().upper()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_method_call_with_args() {
        let result = parse("@name.substr(0, 5)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lambda_single_param() {
        let result = parse("map([1,2,3], x => x * 2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lambda_multi_param() {
        let result = parse("reduce([1,2,3], 0, (acc, x) => acc + x)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lambda_no_param() {
        let result = parse("map([1,2,3], () => 42)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_string_concat() {
        let result = parse("\"hello\" ++ \" world\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_power_operator() {
        let result = parse("2 ** 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_modulo_operator() {
        let result = parse("10 % 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_string_comparison_operators() {
        let result = parse("\"a\" eq \"b\"");
        assert!(result.is_ok());

        let result = parse("\"a\" ne \"b\"");
        assert!(result.is_ok());

        let result = parse("\"a\" lt \"b\"");
        assert!(result.is_ok());

        let result = parse("\"a\" le \"b\"");
        assert!(result.is_ok());

        let result = parse("\"a\" gt \"b\"");
        assert!(result.is_ok());

        let result = parse("\"a\" ge \"b\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_not_operator() {
        let result = parse("not true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_parentheses() {
        let result = parse("(1 + 2) * 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_nested_parentheses() {
        let result = parse("((1 + 2) * 3)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_list() {
        let result = parse("[]");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::List(items) if items.is_empty()));
    }

    #[test]
    fn test_parse_nested_list() {
        let result = parse("[[1, 2], [3, 4]]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_mixed_list() {
        let result = parse("[1, \"hello\", true, null]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_expression() {
        let result = parse("if(true, 1, 0)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_nested_function_calls() {
        let result = parse("upper(trim(\" hello \"))");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_complex_expression() {
        let expr = r#"
            @price * @qty as @total;
            @total * (1 + @tax_rate) as @with_tax;
            @with_tax
        "#;
        let result = parse(expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_comment() {
        let result = parse("1 + 2 // this is a comment");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_multiline_expression() {
        let expr = r#"
            @a + @b;
            @c * @d;
            @e - @f
        "#;
        let result = parse(expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_q_string() {
        let result = parse("q(hello world)");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::String(s) if s == "hello world"));
    }

    #[test]
    fn test_parse_q_string_with_quotes() {
        let result = parse("q(It's a \"test\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let result = parse("1 + * 2");
        assert!(result.is_err());

        let result = parse("@");
        assert!(result.is_err());

        let result = parse("()");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_display() {
        let result = parse("1 + + 2");
        if let Err(e) = result {
            let msg = format!("{}", e);
            assert!(!msg.is_empty());
        }
    }
}
