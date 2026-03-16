use crate::libs::expr::parser::ast::{BinaryOp, Expr};
use crate::libs::expr::parser::builder::util::fold_left;
use crate::libs::expr::parser::ParseError;
use crate::libs::expr::parser::Rule;
use pest::iterators::Pair;

pub fn build_logical_or(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(super::build_expr(item.clone())?);
        } else if item.as_rule() == Rule::op_or {
            ops.push(BinaryOp::Or);
        }
    }

    fold_left(exprs, ops)
}

pub fn build_logical_and(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(super::build_expr(item.clone())?);
        } else if item.as_rule() == Rule::op_and {
            ops.push(BinaryOp::And);
        }
    }

    fold_left(exprs, ops)
}

pub fn build_comparison(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(super::build_expr(item.clone())?);
        } else {
            let op = match item.as_rule() {
                Rule::op_eq => BinaryOp::Eq,
                Rule::op_ne => BinaryOp::Ne,
                Rule::op_lt => BinaryOp::Lt,
                Rule::op_le => BinaryOp::Le,
                Rule::op_gt => BinaryOp::Gt,
                Rule::op_ge => BinaryOp::Ge,
                Rule::op_str_eq => BinaryOp::StrEq,
                Rule::op_str_ne => BinaryOp::StrNe,
                Rule::op_str_lt => BinaryOp::StrLt,
                Rule::op_str_le => BinaryOp::StrLe,
                Rule::op_str_gt => BinaryOp::StrGt,
                Rule::op_str_ge => BinaryOp::StrGe,
                _ => return Err(ParseError::UnexpectedRule(item.as_rule())),
            };
            ops.push(op);
        }
    }

    fold_left(exprs, ops)
}

pub fn build_concat(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();

    for item in inner {
        if item.as_rule() != Rule::op_concat {
            exprs.push(super::build_expr(item)?);
        }
    }

    if exprs.len() == 1 {
        Ok(exprs.into_iter().next().unwrap())
    } else {
        let mut result = exprs[0].clone();
        for expr in exprs.into_iter().skip(1) {
            result = Expr::Binary {
                op: BinaryOp::Concat,
                left: Box::new(result),
                right: Box::new(expr),
            };
        }
        Ok(result)
    }
}

pub fn build_additive(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(super::build_expr(item.clone())?);
        } else {
            let op = match item.as_rule() {
                Rule::op_add => BinaryOp::Add,
                Rule::op_sub => BinaryOp::Sub,
                _ => return Err(ParseError::UnexpectedRule(item.as_rule())),
            };
            ops.push(op);
        }
    }

    fold_left(exprs, ops)
}

pub fn build_multiplicative(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(super::build_expr(item.clone())?);
        } else {
            let op = match item.as_rule() {
                Rule::op_mul => BinaryOp::Mul,
                Rule::op_div => BinaryOp::Div,
                Rule::op_mod => BinaryOp::Mod,
                _ => return Err(ParseError::UnexpectedRule(item.as_rule())),
            };
            ops.push(op);
        }
    }

    fold_left(exprs, ops)
}

pub fn build_power(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(super::build_expr(item.clone())?);
        } else if item.as_rule() == Rule::op_pow {
            ops.push(BinaryOp::Pow);
        }
    }

    fold_left(exprs, ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::expr::parser::parse;

    #[test]
    fn test_logical_or_simple() {
        let expr = parse("true or false").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Or));
                assert!(matches!(left.as_ref(), Expr::Bool(true)));
                assert!(matches!(right.as_ref(), Expr::Bool(false)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_logical_or_multiple() {
        let expr = parse("true or false or true").unwrap();
        // Should be left-associative: (true or false) or true
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Or));
                assert!(matches!(right.as_ref(), Expr::Bool(true)));
                // left should be (true or false)
                match left.as_ref() {
                    Expr::Binary {
                        op: inner_op,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(matches!(inner_op, BinaryOp::Or));
                        assert!(matches!(inner_left.as_ref(), Expr::Bool(true)));
                        assert!(matches!(inner_right.as_ref(), Expr::Bool(false)));
                    }
                    _ => panic!("Expected nested Binary expression"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_logical_and_simple() {
        let expr = parse("true and false").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::And));
                assert!(matches!(left.as_ref(), Expr::Bool(true)));
                assert!(matches!(right.as_ref(), Expr::Bool(false)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_logical_and_multiple() {
        let expr = parse("true and false and true").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::And));
                assert!(matches!(right.as_ref(), Expr::Bool(true)));
                match left.as_ref() {
                    Expr::Binary {
                        op: inner_op,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(matches!(inner_op, BinaryOp::And));
                        assert!(matches!(inner_left.as_ref(), Expr::Bool(true)));
                        assert!(matches!(inner_right.as_ref(), Expr::Bool(false)));
                    }
                    _ => panic!("Expected nested Binary expression"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_comparison_eq() {
        let expr = parse("1 == 2").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Eq));
                assert!(matches!(left.as_ref(), Expr::Int(1)));
                assert!(matches!(right.as_ref(), Expr::Int(2)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_comparison_ne() {
        let expr = parse("1 != 2").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Ne));
                assert!(matches!(left.as_ref(), Expr::Int(1)));
                assert!(matches!(right.as_ref(), Expr::Int(2)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_comparison_lt() {
        let expr = parse("1 < 2").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Lt));
                assert!(matches!(left.as_ref(), Expr::Int(1)));
                assert!(matches!(right.as_ref(), Expr::Int(2)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_comparison_le() {
        let expr = parse("1 <= 2").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Le));
                assert!(matches!(left.as_ref(), Expr::Int(1)));
                assert!(matches!(right.as_ref(), Expr::Int(2)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_comparison_gt() {
        let expr = parse("1 > 2").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Gt));
                assert!(matches!(left.as_ref(), Expr::Int(1)));
                assert!(matches!(right.as_ref(), Expr::Int(2)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_comparison_ge() {
        let expr = parse("1 >= 2").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Ge));
                assert!(matches!(left.as_ref(), Expr::Int(1)));
                assert!(matches!(right.as_ref(), Expr::Int(2)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_comparison_chain_not_supported() {
        // Chain comparisons like "1 < 2 < 3" are not supported in the grammar
        // Each comparison must be explicit: "1 < 2 and 2 < 3"
        let result = parse("1 < 2 < 3");
        assert!(result.is_err());
    }

    #[test]
    fn test_concat_simple() {
        let expr = parse("\"hello\" ++ \"world\"").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Concat));
                assert!(matches!(left.as_ref(), Expr::String(s) if s == "hello"));
                assert!(matches!(right.as_ref(), Expr::String(s) if s == "world"));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_concat_multiple() {
        let expr = parse("\"a\" ++ \"b\" ++ \"c\"").unwrap();
        // Should be left-associative: ("a" ++ "b") ++ "c"
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Concat));
                assert!(matches!(right.as_ref(), Expr::String(s) if s == "c"));
                match left.as_ref() {
                    Expr::Binary {
                        op: inner_op,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(matches!(inner_op, BinaryOp::Concat));
                        assert!(
                            matches!(inner_left.as_ref(), Expr::String(s) if s == "a")
                        );
                        assert!(
                            matches!(inner_right.as_ref(), Expr::String(s) if s == "b")
                        );
                    }
                    _ => panic!("Expected nested Binary expression"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_additive_add() {
        let expr = parse("1 + 2").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Add));
                assert!(matches!(left.as_ref(), Expr::Int(1)));
                assert!(matches!(right.as_ref(), Expr::Int(2)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_additive_sub() {
        let expr = parse("1 - 2").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Sub));
                assert!(matches!(left.as_ref(), Expr::Int(1)));
                assert!(matches!(right.as_ref(), Expr::Int(2)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_additive_chain() {
        let expr = parse("1 + 2 - 3").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Sub));
                assert!(matches!(right.as_ref(), Expr::Int(3)));
                match left.as_ref() {
                    Expr::Binary {
                        op: inner_op,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(matches!(inner_op, BinaryOp::Add));
                        assert!(matches!(inner_left.as_ref(), Expr::Int(1)));
                        assert!(matches!(inner_right.as_ref(), Expr::Int(2)));
                    }
                    _ => panic!("Expected nested Binary expression"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_multiplicative_mul() {
        let expr = parse("2 * 3").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Mul));
                assert!(matches!(left.as_ref(), Expr::Int(2)));
                assert!(matches!(right.as_ref(), Expr::Int(3)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_multiplicative_div() {
        let expr = parse("6 / 3").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Div));
                assert!(matches!(left.as_ref(), Expr::Int(6)));
                assert!(matches!(right.as_ref(), Expr::Int(3)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_multiplicative_mod() {
        let expr = parse("7 % 3").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Mod));
                assert!(matches!(left.as_ref(), Expr::Int(7)));
                assert!(matches!(right.as_ref(), Expr::Int(3)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_multiplicative_chain() {
        let expr = parse("2 * 3 / 4").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Div));
                assert!(matches!(right.as_ref(), Expr::Int(4)));
                match left.as_ref() {
                    Expr::Binary {
                        op: inner_op,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(matches!(inner_op, BinaryOp::Mul));
                        assert!(matches!(inner_left.as_ref(), Expr::Int(2)));
                        assert!(matches!(inner_right.as_ref(), Expr::Int(3)));
                    }
                    _ => panic!("Expected nested Binary expression"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_power_simple() {
        let expr = parse("2 ** 3").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Pow));
                assert!(matches!(left.as_ref(), Expr::Int(2)));
                assert!(matches!(right.as_ref(), Expr::Int(3)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_power_chain() {
        let expr = parse("2 ** 3 ** 2").unwrap();
        // Should be left-associative: (2 ** 3) ** 2
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Pow));
                assert!(matches!(right.as_ref(), Expr::Int(2)));
                match left.as_ref() {
                    Expr::Binary {
                        op: inner_op,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(matches!(inner_op, BinaryOp::Pow));
                        assert!(matches!(inner_left.as_ref(), Expr::Int(2)));
                        assert!(matches!(inner_right.as_ref(), Expr::Int(3)));
                    }
                    _ => panic!("Expected nested Binary expression"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        // 1 + 2 * 3 should be 1 + (2 * 3)
        let expr = parse("1 + 2 * 3").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Add));
                assert!(matches!(left.as_ref(), Expr::Int(1)));
                // right should be (2 * 3)
                match right.as_ref() {
                    Expr::Binary {
                        op: inner_op,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(matches!(inner_op, BinaryOp::Mul));
                        assert!(matches!(inner_left.as_ref(), Expr::Int(2)));
                        assert!(matches!(inner_right.as_ref(), Expr::Int(3)));
                    }
                    _ => panic!("Expected nested Binary expression for multiplication"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_operator_precedence_with_power() {
        // 2 * 3 ** 2 should be 2 * (3 ** 2)
        let expr = parse("2 * 3 ** 2").unwrap();
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Mul));
                assert!(matches!(left.as_ref(), Expr::Int(2)));
                match right.as_ref() {
                    Expr::Binary {
                        op: inner_op,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(matches!(inner_op, BinaryOp::Pow));
                        assert!(matches!(inner_left.as_ref(), Expr::Int(3)));
                        assert!(matches!(inner_right.as_ref(), Expr::Int(2)));
                    }
                    _ => panic!("Expected nested Binary expression for power"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_string_comparison_operators() {
        // Test string comparison operators: eq, ne, lt, le, gt, ge
        let expr = parse("\"a\" eq \"b\"").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert!(matches!(op, BinaryOp::StrEq));
            }
            _ => panic!("Expected Binary expression"),
        }

        let expr = parse("\"a\" ne \"b\"").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert!(matches!(op, BinaryOp::StrNe));
            }
            _ => panic!("Expected Binary expression"),
        }

        let expr = parse("\"a\" lt \"b\"").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert!(matches!(op, BinaryOp::StrLt));
            }
            _ => panic!("Expected Binary expression"),
        }

        let expr = parse("\"a\" le \"b\"").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert!(matches!(op, BinaryOp::StrLe));
            }
            _ => panic!("Expected Binary expression"),
        }

        let expr = parse("\"a\" gt \"b\"").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert!(matches!(op, BinaryOp::StrGt));
            }
            _ => panic!("Expected Binary expression"),
        }

        let expr = parse("\"a\" ge \"b\"").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert!(matches!(op, BinaryOp::StrGe));
            }
            _ => panic!("Expected Binary expression"),
        }
    }
}
