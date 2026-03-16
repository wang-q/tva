use crate::libs::expr::parser::ast::{Expr, UnaryOp};
use crate::libs::expr::parser::ParseError;
use crate::libs::expr::parser::Rule;
use pest::iterators::Pair;

pub fn build_unary(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut ops: Vec<UnaryOp> = Vec::new();
    let mut inner_pairs: Vec<Pair<Rule>> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::op_not => ops.push(UnaryOp::Not),
            Rule::op_neg => ops.push(UnaryOp::Neg),
            _ => inner_pairs.push(inner),
        }
    }

    // Build the base expression from inner pairs (postfix content)
    // Handle method chain: primary/func_call followed by method_call(s)
    let mut result = if inner_pairs.is_empty() {
        return Err(ParseError::EmptyExpression);
    } else {
        // First pair is the base (primary or func_call)
        let first = &inner_pairs[0];
        let base = match first.as_rule() {
            Rule::func_call => super::build_func_call(first.clone())?,
            Rule::postfix => super::build_postfix(first.clone())?,
            Rule::primary => super::build_primary(first.clone())?,
            _ => super::build_expr(first.clone())?,
        };

        // Remaining pairs are method calls
        let mut obj = base;
        for method_pair in inner_pairs.iter().skip(1) {
            if method_pair.as_rule() == Rule::method_call {
                let (name, args) = super::build_method_call(method_pair.clone())?;
                obj = Expr::MethodCall {
                    object: Box::new(obj),
                    name,
                    args,
                };
            }
        }
        obj
    };

    for op in ops.into_iter().rev() {
        result = Expr::Unary {
            op,
            expr: Box::new(result),
        };
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::expr::parser::ast::ColumnRef;
    use crate::libs::expr::parser::parse;

    #[test]
    fn test_unary_neg() {
        let expr = parse("-5").unwrap();
        match expr {
            Expr::Unary { op, expr } => {
                assert!(matches!(op, UnaryOp::Neg));
                assert!(matches!(expr.as_ref(), Expr::Int(5)));
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_unary_not() {
        let expr = parse("not true").unwrap();
        match expr {
            Expr::Unary { op, expr } => {
                assert!(matches!(op, UnaryOp::Not));
                assert!(matches!(expr.as_ref(), Expr::Bool(true)));
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_unary_double_neg() {
        let expr = parse("--5").unwrap();
        match expr {
            Expr::Unary {
                op: outer_op,
                expr: outer_expr,
            } => {
                assert!(matches!(outer_op, UnaryOp::Neg));
                match outer_expr.as_ref() {
                    Expr::Unary {
                        op: inner_op,
                        expr: inner_expr,
                    } => {
                        assert!(matches!(inner_op, UnaryOp::Neg));
                        assert!(matches!(inner_expr.as_ref(), Expr::Int(5)));
                    }
                    _ => panic!("Expected nested Unary expression"),
                }
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_unary_not_not() {
        let expr = parse("not not true").unwrap();
        match expr {
            Expr::Unary {
                op: outer_op,
                expr: outer_expr,
            } => {
                assert!(matches!(outer_op, UnaryOp::Not));
                match outer_expr.as_ref() {
                    Expr::Unary {
                        op: inner_op,
                        expr: inner_expr,
                    } => {
                        assert!(matches!(inner_op, UnaryOp::Not));
                        assert!(matches!(inner_expr.as_ref(), Expr::Bool(true)));
                    }
                    _ => panic!("Expected nested Unary expression"),
                }
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_unary_mixed() {
        let expr = parse("not -5").unwrap();
        match expr {
            Expr::Unary { op, expr } => {
                assert!(matches!(op, UnaryOp::Not));
                match expr.as_ref() {
                    Expr::Unary {
                        op: inner_op,
                        expr: inner_expr,
                    } => {
                        assert!(matches!(inner_op, UnaryOp::Neg));
                        assert!(matches!(inner_expr.as_ref(), Expr::Int(5)));
                    }
                    _ => panic!("Expected nested Unary expression"),
                }
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_unary_with_column_ref() {
        let expr = parse("-@price").unwrap();
        match expr {
            Expr::Unary { op, expr } => {
                assert!(matches!(op, UnaryOp::Neg));
                match expr.as_ref() {
                    Expr::ColumnRef(ColumnRef::Name(name)) => {
                        assert_eq!(name, "price");
                    }
                    _ => panic!("Expected ColumnRef::Name expression"),
                }
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_unary_not_with_comparison() {
        // "not" has higher precedence than comparison, so "not @1 > 5" parses as "(not @1) > 5"
        let expr = parse("not @1 > 5").unwrap();
        match expr {
            Expr::Binary {
                op: bin_op, left, ..
            } => {
                assert!(matches!(
                    bin_op,
                    crate::libs::expr::parser::ast::BinaryOp::Gt
                ));
                // left side should be "not @1"
                match left.as_ref() {
                    Expr::Unary { op, expr } => {
                        assert!(matches!(op, UnaryOp::Not));
                        assert!(matches!(
                            expr.as_ref(),
                            Expr::ColumnRef(ColumnRef::Index(1))
                        ));
                    }
                    _ => {
                        panic!("Expected Unary expression on left side, got {:?}", left)
                    }
                }
            }
            _ => panic!("Expected Binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_unary_neg_with_float() {
        let expr = parse("-3.14").unwrap();
        match expr {
            Expr::Unary { op, expr } => {
                assert!(matches!(op, UnaryOp::Neg));
                assert!(
                    matches!(expr.as_ref(), Expr::Float(f) if (f - 3.14).abs() < 0.001)
                );
            }
            _ => panic!("Expected Unary expression"),
        }
    }
}
