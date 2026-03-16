use crate::libs::expr::parser::ast::{BinaryOp, Expr};
use crate::libs::expr::parser::ParseError;

pub fn fold_left(exprs: Vec<Expr>, ops: Vec<BinaryOp>) -> Result<Expr, ParseError> {
    if exprs.is_empty() {
        return Err(ParseError::EmptyExpression);
    }
    if exprs.len() == 1 {
        return Ok(exprs.into_iter().next().unwrap());
    }

    let mut result = exprs[0].clone();
    for (i, op) in ops.iter().enumerate() {
        result = Expr::Binary {
            op: *op,
            left: Box::new(result),
            right: Box::new(exprs[i + 1].clone()),
        };
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_left_single_expr() {
        let exprs = vec![Expr::Int(42)];
        let ops = vec![];
        let result = fold_left(exprs, ops).unwrap();
        assert!(matches!(result, Expr::Int(42)));
    }

    #[test]
    fn test_fold_left_two_exprs() {
        let exprs = vec![Expr::Int(1), Expr::Int(2)];
        let ops = vec![BinaryOp::Add];
        let result = fold_left(exprs, ops).unwrap();
        match result {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Add));
                assert!(matches!(left.as_ref(), Expr::Int(1)));
                assert!(matches!(right.as_ref(), Expr::Int(2)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_fold_left_three_exprs() {
        // (1 + 2) + 3
        let exprs = vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)];
        let ops = vec![BinaryOp::Add, BinaryOp::Add];
        let result = fold_left(exprs, ops).unwrap();
        match result {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Add));
                assert!(matches!(right.as_ref(), Expr::Int(3)));
                // left should be (1 + 2)
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
    fn test_fold_left_empty_exprs() {
        let exprs: Vec<Expr> = vec![];
        let ops = vec![];
        let result = fold_left(exprs, ops);
        assert!(result.is_err());
    }

    #[test]
    fn test_fold_left_mixed_ops() {
        // 1 + 2 - 3 => (1 + 2) - 3
        let exprs = vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)];
        let ops = vec![BinaryOp::Add, BinaryOp::Sub];
        let result = fold_left(exprs, ops).unwrap();
        match result {
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
    fn test_fold_left_with_strings() {
        let exprs = vec![
            Expr::String("hello".to_string()),
            Expr::String(" ".to_string()),
            Expr::String("world".to_string()),
        ];
        let ops = vec![BinaryOp::Concat, BinaryOp::Concat];
        let result = fold_left(exprs, ops).unwrap();
        match result {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Concat));
                assert!(matches!(right.as_ref(), Expr::String(s) if s == "world"));
            }
            _ => panic!("Expected Binary expression"),
        }
    }
}
