use super::ast::{ColumnRef, Expr};
use super::ParseError;
use pest::iterators::Pair;

mod binary;
mod expr;
mod lambda;
mod literal;
mod postfix;
mod primary;
mod unary;
mod util;

pub use binary::*;
pub use expr::*;
pub use lambda::*;
pub use literal::*;
pub use postfix::*;
pub use primary::*;
pub use unary::*;

/// Build expression from a pair
pub fn build_expr(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    match pair.as_rule() {
        super::Rule::full_expr => build_full_expr(pair),
        super::Rule::expr_list => build_full_expr(pair),
        super::Rule::expr => {
            let mut inner = pair.into_inner();
            let bind_pair = inner.next().ok_or(ParseError::EmptyExpression)?;
            build_expr(bind_pair)
        }
        super::Rule::bind => build_bind(pair),
        super::Rule::pipe => build_pipe(pair),
        super::Rule::logical_or => build_logical_or(pair),
        super::Rule::logical_and => build_logical_and(pair),
        super::Rule::comparison => build_comparison(pair),
        super::Rule::concat => build_concat(pair),
        super::Rule::additive => build_additive(pair),
        super::Rule::multiplicative => build_multiplicative(pair),
        super::Rule::power => build_power(pair),
        super::Rule::unary => build_unary(pair),
        super::Rule::postfix => build_postfix(pair),
        super::Rule::primary => build_primary(pair),
        super::Rule::func_call => build_func_call(pair),
        super::Rule::ident_or_lambda => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or(ParseError::EmptyExpression)?;
            build_expr(inner)
        }
        super::Rule::lambda_single_param => build_lambda(pair),
        super::Rule::lambda_multi_params => build_lambda(pair),
        super::Rule::method_call => {
            let (name, args) = build_method_call(pair)?;
            Ok(Expr::Call { name, args })
        }
        super::Rule::list_literal => build_list_literal(pair),
        super::Rule::column_ref => build_column_ref(pair.as_str()),
        super::Rule::variable_ref => build_variable_ref(pair.as_str()),
        super::Rule::string => build_string(pair.as_str()),
        super::Rule::q_string => build_string(pair.as_str()),
        super::Rule::double_quoted_string => build_string(pair.as_str()),
        super::Rule::single_quoted_string => build_string(pair.as_str()),
        super::Rule::float => {
            let num: f64 = pair
                .as_str()
                .parse()
                .map_err(|_| ParseError::InvalidNumber(pair.as_str().to_string()))?;
            Ok(Expr::Float(num))
        }
        super::Rule::int => {
            let num: i64 = pair
                .as_str()
                .parse()
                .map_err(|_| ParseError::InvalidNumber(pair.as_str().to_string()))?;
            Ok(Expr::Int(num))
        }
        super::Rule::boolean => {
            let b = pair.as_str() == "true";
            Ok(Expr::Bool(b))
        }
        super::Rule::null => Ok(Expr::Null),
        super::Rule::ident => {
            let name = pair.as_str();
            if name == "_" {
                Ok(Expr::Underscore)
            } else {
                Ok(Expr::ColumnRef(ColumnRef::Name(name.to_string())))
            }
        }
        _ => Err(ParseError::UnexpectedRule(pair.as_rule())),
    }
}

#[cfg(test)]
mod tests {
    use super::super::ast::{BinaryOp, ColumnRef, Expr, PipeRight, UnaryOp};
    use super::super::parse;

    #[test]
    fn test_parse_int() {
        let expr = parse("123").unwrap();
        assert!(matches!(expr, Expr::Int(123)));
    }

    #[test]
    fn test_parse_float() {
        // Decimal notation
        let expr = parse("3.14").unwrap();
        assert!(matches!(expr, Expr::Float(n) if (n - 3.14).abs() < 0.001));

        let expr = parse("-0.5").unwrap();
        match expr {
            Expr::Unary {
                op: UnaryOp::Neg,
                expr,
            } => {
                assert!(matches!(*expr, Expr::Float(n) if (n - 0.5).abs() < 0.001));
            }
            _ => panic!("Expected Neg unary expression for negative float"),
        }

        // Scientific notation: 1e10, 2.5e-3, -1.5E+6
        let expr = parse("1e10").unwrap();
        assert!(matches!(expr, Expr::Float(n) if (n - 1e10).abs() < 0.1));

        let expr = parse("2.5e-3").unwrap();
        assert!(matches!(expr, Expr::Float(n) if (n - 0.0025).abs() < 0.0001));

        let expr = parse("-1.5e6").unwrap();
        match expr {
            Expr::Unary {
                op: UnaryOp::Neg,
                expr,
            } => {
                assert!(matches!(*expr, Expr::Float(n) if (n - 1.5e6).abs() < 0.1));
            }
            _ => {
                panic!("Expected Neg unary expression for negative scientific notation")
            }
        }
    }

    #[test]
    fn test_parse_bool() {
        let expr = parse("true").unwrap();
        assert!(matches!(expr, Expr::Bool(true)));

        let expr = parse("false").unwrap();
        assert!(matches!(expr, Expr::Bool(false)));
    }

    #[test]
    fn test_parse_null() {
        let expr = parse("null").unwrap();
        assert!(matches!(expr, Expr::Null));
    }

    #[test]
    fn test_parse_addition() {
        let expr = parse("@1 + @2").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::Add,
                left,
                right,
            } => {
                assert!(matches!(*left, Expr::ColumnRef(ColumnRef::Index(1))));
                assert!(matches!(*right, Expr::ColumnRef(ColumnRef::Index(2))));
            }
            _ => panic!("Expected Add expression"),
        }
    }

    #[test]
    fn test_parse_precedence() {
        let expr = parse("@1 + @2 * 3").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::Add,
                left,
                right,
            } => {
                assert!(matches!(*left, Expr::ColumnRef(ColumnRef::Index(1))));
                match *right {
                    Expr::Binary {
                        op: BinaryOp::Mul,
                        left,
                        right,
                    } => {
                        assert!(matches!(*left, Expr::ColumnRef(ColumnRef::Index(2))));
                        assert!(matches!(*right, Expr::Int(3)));
                    }
                    _ => panic!("Expected Mul on right side"),
                }
            }
            _ => panic!("Expected Add expression with precedence"),
        }
    }

    #[test]
    fn test_parse_parentheses() {
        let expr = parse("(@1 + @2) * 3").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::Mul,
                left,
                right,
            } => {
                match *left {
                    Expr::Binary {
                        op: BinaryOp::Add, ..
                    } => {}
                    _ => panic!("Expected Add inside parentheses"),
                }
                assert!(matches!(*right, Expr::Int(3)));
            }
            _ => panic!("Expected Mul expression"),
        }
    }

    #[test]
    fn test_parse_comparison() {
        let expr = parse("@1 > 10").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::Gt, ..
            } => {}
            _ => panic!("Expected Gt expression"),
        }
    }

    #[test]
    fn test_parse_logical() {
        let expr = parse("@1 > 0 and @2 < 100").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::And, ..
            } => {}
            _ => panic!("Expected And expression"),
        }
    }

    #[test]
    fn test_parse_logical_word() {
        let expr = parse("@1 > 0 and @2 < 100").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::And, ..
            } => {}
            _ => panic!("Expected And expression with 'and' keyword"),
        }
    }

    #[test]
    fn test_parse_unary() {
        let expr = parse("-@1").unwrap();
        match expr {
            Expr::Unary {
                op: UnaryOp::Neg, ..
            } => {}
            _ => panic!("Expected Neg unary expression"),
        }
    }

    #[test]
    fn test_parse_negative_number() {
        let expr = parse("-42").unwrap();
        match expr {
            Expr::Unary {
                op: UnaryOp::Neg,
                expr,
            } => {
                assert!(matches!(*expr, Expr::Int(42)));
            }
            _ => panic!("Expected Neg unary expression for negative int"),
        }

        let expr = parse("-3.14").unwrap();
        match expr {
            Expr::Unary {
                op: UnaryOp::Neg,
                expr,
            } => {
                assert!(matches!(*expr, Expr::Float(n) if (n - 3.14).abs() < 0.001));
            }
            _ => panic!("Expected Neg unary expression for negative float"),
        }
    }

    #[test]
    fn test_parse_not_operator() {
        // 'not' keyword is supported
        let expr = parse("not @valid").unwrap();
        match expr {
            Expr::Unary {
                op: UnaryOp::Not,
                expr,
            } => {
                assert!(
                    matches!(*expr, Expr::ColumnRef(ColumnRef::Name(s)) if s == "valid")
                );
            }
            _ => panic!("Expected Not unary expression with 'not' keyword"),
        }

        // '!' operator is not supported by grammar
        assert!(parse("!true").is_err());
    }

    #[test]
    fn test_parse_all_operators() {
        // Arithmetic
        assert!(matches!(
            parse("@1 - @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Sub,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 * @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 / @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Div,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 % @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Mod,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 ** @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Pow,
                ..
            }
        ));

        // Comparison
        assert!(matches!(
            parse("@1 == @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Eq,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 != @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Ne,
                ..
            }
        ));
        // <> is not supported
        assert!(parse("@1 <> @2").is_err());
        assert!(matches!(
            parse("@1 < @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Lt,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 <= @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Le,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 > @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Gt,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 >= @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Ge,
                ..
            }
        ));

        // String comparison
        assert!(matches!(
            parse("@1 eq @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::StrEq,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 ne @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::StrNe,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 lt @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::StrLt,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 le @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::StrLe,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 gt @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::StrGt,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 ge @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::StrGe,
                ..
            }
        ));

        // Logical - using word operators
        assert!(matches!(
            parse("@1 and @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::And,
                ..
            }
        ));
        assert!(matches!(
            parse("@1 or @2").unwrap(),
            Expr::Binary {
                op: BinaryOp::Or,
                ..
            }
        ));

        // Symbol operators && and || may not be supported
        assert!(parse("@1 && @2").is_err());
        assert!(parse("@1 || @2").is_err());
    }

    #[test]
    fn test_parse_string_concat() {
        let expr = parse("@first ++ \" \" ++ @last").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::Concat,
                ..
            } => {}
            _ => panic!("Expected Concat expression"),
        }
    }

    #[test]
    fn test_parse_variable_bind() {
        let expr = parse("@price * @qty as @total").unwrap();
        match expr {
            Expr::Bind { name, .. } => {
                assert_eq!(name, "total");
            }
            _ => panic!("Expected Bind expression"),
        }
    }

    #[test]
    fn test_parse_pipe() {
        // @name | trim() | upper() should parse as ((@name | trim()) | upper())
        let expr = parse("@name | trim() | upper()").unwrap();
        match expr {
            Expr::Pipe { left, right } => {
                // left should be (@name | trim())
                match *left {
                    Expr::Pipe {
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(
                            matches!(*inner_left, Expr::ColumnRef(ColumnRef::Name(s)) if s == "name")
                        );
                        match *inner_right {
                            PipeRight::Call { name, .. } => assert_eq!(name, "trim"),
                            _ => panic!("Expected Call pipe right for trim"),
                        }
                    }
                    _ => panic!("Expected nested Pipe expression for left side"),
                }
                // right should be upper()
                match *right {
                    PipeRight::Call { name, .. } => assert_eq!(name, "upper"),
                    _ => panic!("Expected Call pipe right for upper"),
                }
            }
            _ => panic!("Expected Pipe expression"),
        }
    }

    #[test]
    fn test_parse_pipe_with_placeholder() {
        let expr = parse("@desc | substr(_, 0, 50)").unwrap();
        match expr {
            Expr::Pipe { right, .. } => match *right {
                PipeRight::CallWithPlaceholder { name, .. } => {
                    assert_eq!(name, "substr")
                }
                _ => panic!("Expected CallWithPlaceholder"),
            },
            _ => panic!("Expected Pipe expression"),
        }
    }

    #[test]
    fn test_parse_multiple_exprs() {
        let expr = parse("@price as @p; @qty as @q; @p * @q").unwrap();
        match expr {
            Expr::Block(exprs) => {
                assert_eq!(exprs.len(), 3);
            }
            _ => panic!("Expected Block expression"),
        }
    }

    #[test]
    fn test_parse_comment() {
        // Comments should be ignored
        let expr = parse("@1 + @2 // this is a comment").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::Add, ..
            } => {}
            _ => panic!("Expected Add expression"),
        }
    }

    #[test]
    fn test_parse_errors() {
        assert!(parse("@").is_err());
        // @0 is now valid (refers to whole row)
        assert!(parse("").is_err());
    }

    #[test]
    fn test_parse_function_call() {
        // Basic function call
        let expr = parse("trim(@name)").unwrap();
        match expr {
            Expr::Call { name, args } => {
                assert_eq!(name, "trim");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_parse_function_call_no_args() {
        let expr = parse("now()").unwrap();
        match expr {
            Expr::Call { name, args } => {
                assert_eq!(name, "now");
                assert!(args.is_empty());
            }
            _ => panic!("Expected Call with no args"),
        }
    }

    #[test]
    fn test_parse_function_call_multiple_args() {
        let expr = parse("substr(@name, 0, 5)").unwrap();
        match expr {
            Expr::Call { name, args } => {
                assert_eq!(name, "substr");
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected Call with multiple args"),
        }
    }
}
