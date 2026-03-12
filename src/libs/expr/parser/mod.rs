pub mod ast;

use ast::{BinaryOp, ColumnRef, Expr, UnaryOp};
use pest::iterators::Pair;
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
}

/// Parse an expression string into an AST
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let pairs = ExprParser::parse(Rule::full_expr, input)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::full_expr | Rule::expr => {
                return build_expr(pair);
            }
            _ => {}
        }
    }
    Err(ParseError::InvalidNumber("empty input".to_string()))
}

/// Build expression from a pair
fn build_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    match pair.as_rule() {
        Rule::full_expr => {
            // full_expr has one child: expr
            let inner = pair.into_inner().next().ok_or_else(|| {
                ParseError::InvalidNumber("empty full_expr".to_string())
            })?;
            build_expr(inner)
        }
        Rule::expr => {
            // expr is logical_or
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::InvalidNumber("empty expr".to_string()))?;
            build_expr(inner)
        }
        Rule::logical_or => build_logical_or(pair),
        Rule::logical_and => build_logical_and(pair),
        Rule::comparison => build_comparison(pair),
        Rule::additive => build_additive(pair),
        Rule::multiplicative => build_multiplicative(pair),
        Rule::power => build_power(pair),
        Rule::unary => build_unary(pair),
        Rule::postfix => build_postfix(pair),
        Rule::primary => build_primary(pair),
        Rule::func_call => build_func_call(pair),
        Rule::column_ref => build_column_ref(pair.as_str()),
        Rule::string => build_string(pair.as_str()),
        Rule::float => {
            let num: f64 = pair
                .as_str()
                .parse()
                .map_err(|_| ParseError::InvalidNumber(pair.as_str().to_string()))?;
            Ok(Expr::Float(num))
        }
        Rule::int => {
            let num: i64 = pair
                .as_str()
                .parse()
                .map_err(|_| ParseError::InvalidNumber(pair.as_str().to_string()))?;
            Ok(Expr::Int(num))
        }
        Rule::boolean => {
            let b = pair.as_str() == "true";
            Ok(Expr::Bool(b))
        }
        Rule::null => Ok(Expr::Null),
        Rule::ident => Ok(Expr::ColumnRef(ColumnRef::Name(pair.as_str().to_string()))),
        _ => Err(ParseError::UnexpectedRule(pair.as_rule())),
    }
}

fn build_logical_or(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut exprs: Vec<Expr> = Vec::new();
    let mut ops: Vec<BinaryOp> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::op_or => ops.push(BinaryOp::Or),
            _ => exprs.push(build_expr(inner)?),
        }
    }

    fold_left(exprs, ops)
}

fn build_logical_and(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut exprs: Vec<Expr> = Vec::new();
    let mut ops: Vec<BinaryOp> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::op_and => ops.push(BinaryOp::And),
            _ => exprs.push(build_expr(inner)?),
        }
    }

    fold_left(exprs, ops)
}

fn build_comparison(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut exprs: Vec<Expr> = Vec::new();
    let mut ops: Vec<BinaryOp> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::op_eq
            | Rule::op_ne
            | Rule::op_lt
            | Rule::op_le
            | Rule::op_gt
            | Rule::op_ge => {
                // Check the actual operator string
                let s = inner.as_str();
                match s {
                    "==" | "=" => ops.push(BinaryOp::Eq),
                    "!=" | "<>" => ops.push(BinaryOp::Ne),
                    "<" => ops.push(BinaryOp::Lt),
                    "<=" => ops.push(BinaryOp::Le),
                    ">" => ops.push(BinaryOp::Gt),
                    ">=" => ops.push(BinaryOp::Ge),
                    _ => {
                        return Err(ParseError::InvalidNumber(format!(
                            "unknown comparison op: {}",
                            s
                        )))
                    }
                }
            }
            _ => exprs.push(build_expr(inner)?),
        }
    }

    fold_left(exprs, ops)
}

fn build_additive(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut exprs: Vec<Expr> = Vec::new();
    let mut ops: Vec<BinaryOp> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::op_add => ops.push(BinaryOp::Add),
            Rule::op_sub => ops.push(BinaryOp::Sub),
            _ => exprs.push(build_expr(inner)?),
        }
    }

    fold_left(exprs, ops)
}

fn build_multiplicative(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut exprs: Vec<Expr> = Vec::new();
    let mut ops: Vec<BinaryOp> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::op_mul => ops.push(BinaryOp::Mul),
            Rule::op_div => ops.push(BinaryOp::Div),
            Rule::op_mod => ops.push(BinaryOp::Mod),
            _ => exprs.push(build_expr(inner)?),
        }
    }

    fold_left(exprs, ops)
}

fn build_power(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut exprs: Vec<Expr> = Vec::new();
    let mut ops: Vec<BinaryOp> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::op_pow => ops.push(BinaryOp::Pow),
            _ => exprs.push(build_expr(inner)?),
        }
    }

    fold_left(exprs, ops)
}

fn build_unary(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut ops: Vec<UnaryOp> = Vec::new();
    let mut inner_expr: Option<Expr> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::op_not => ops.push(UnaryOp::Not),
            Rule::op_neg => ops.push(UnaryOp::Neg),
            _ => inner_expr = Some(build_expr(inner)?),
        }
    }

    let mut result = inner_expr
        .ok_or_else(|| ParseError::InvalidNumber("empty unary".to_string()))?;

    // Apply unary operators right to left
    for op in ops.into_iter().rev() {
        result = Expr::unary(op, result);
    }

    Ok(result)
}

fn build_postfix(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    for inner in pair.into_inner() {
        return build_expr(inner);
    }
    Err(ParseError::InvalidNumber("empty postfix".to_string()))
}

fn build_func_call(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut name = String::new();
    let mut args: Vec<Expr> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => name = inner.as_str().to_string(),
            Rule::expr => args.push(build_expr(inner)?),
            _ => {}
        }
    }

    Ok(Expr::call(name, args))
}

fn build_primary(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    for inner in pair.into_inner() {
        return build_expr(inner);
    }
    Err(ParseError::InvalidNumber("empty primary".to_string()))
}

fn build_column_ref(s: &str) -> Result<Expr, ParseError> {
    // s is like "@1" or "@name"
    let inner = &s[1..]; // Remove '@'

    if inner
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        let idx: usize = inner
            .parse()
            .map_err(|_| ParseError::InvalidColumnIndex(inner.to_string()))?;
        if idx == 0 {
            return Err(ParseError::InvalidColumnIndex("0".to_string()));
        }
        Ok(Expr::ColumnRef(ColumnRef::Index(idx)))
    } else {
        Ok(Expr::ColumnRef(ColumnRef::Name(inner.to_string())))
    }
}

fn build_string(s: &str) -> Result<Expr, ParseError> {
    // Remove surrounding quotes
    let inner = &s[1..s.len() - 1];
    Ok(Expr::String(inner.to_string()))
}

fn fold_left(exprs: Vec<Expr>, ops: Vec<BinaryOp>) -> Result<Expr, ParseError> {
    if exprs.is_empty() {
        return Err(ParseError::InvalidNumber("empty expression".to_string()));
    }
    if exprs.len() == 1 {
        return Ok(exprs.into_iter().next().unwrap());
    }

    let mut result = exprs[0].clone();
    for (i, op) in ops.iter().enumerate() {
        result = Expr::binary(*op, result, exprs[i + 1].clone());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_column_ref_index() {
        let expr = parse("@1").unwrap();
        assert!(matches!(expr, Expr::ColumnRef(ColumnRef::Index(1))));
    }

    #[test]
    fn test_parse_column_ref_name() {
        let expr = parse("@price").unwrap();
        assert!(matches!(expr, Expr::ColumnRef(ColumnRef::Name(s)) if s == "price"));
    }

    #[test]
    fn test_parse_int() {
        let expr = parse("123").unwrap();
        assert!(matches!(expr, Expr::Int(123)));
    }

    #[test]
    fn test_parse_float() {
        let expr = parse("3.14").unwrap();
        assert!(matches!(expr, Expr::Float(n) if (n - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_parse_string() {
        let expr = parse("\"hello\"").unwrap();
        assert!(matches!(expr, Expr::String(s) if s == "hello"));
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
        // @1 + @2 * 3 should be @1 + (@2 * 3)
        let expr = parse("@1 + @2 * 3").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::Add,
                left,
                right,
            } => {
                assert!(matches!(*left, Expr::ColumnRef(ColumnRef::Index(1))));
                // Right side should be @2 * 3
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
                // Left should be @1 + @2
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
        let expr = parse("@1 > 0 && @2 < 100").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::And, ..
            } => {}
            _ => panic!("Expected And expression"),
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
    fn test_parse_errors() {
        // Empty column reference
        assert!(parse("@").is_err());

        // Invalid column index (0)
        assert!(parse("@0").is_err());

        // Unexpected character
        assert!(parse("@1 + $").is_err());
    }
}
