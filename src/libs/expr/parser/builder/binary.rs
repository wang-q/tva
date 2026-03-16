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
