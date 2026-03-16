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
