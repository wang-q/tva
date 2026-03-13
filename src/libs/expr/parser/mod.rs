pub mod ast;

use ast::{BinaryOp, ColumnRef, Expr, PipeRight, UnaryOp};
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

/// Build full expression (handles multiple expressions separated by semicolons)
fn build_full_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut exprs = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr_list => {
                for expr_pair in inner.into_inner() {
                    match expr_pair.as_rule() {
                        Rule::expr => {
                            exprs.push(build_expr(expr_pair)?);
                        }
                        _ => {}
                    }
                }
            }
            Rule::expr => {
                exprs.push(build_expr(inner)?);
            }
            _ => {}
        }
    }

    match exprs.len() {
        0 => Err(ParseError::EmptyExpression),
        1 => Ok(exprs.into_iter().next().unwrap()),
        _ => Ok(Expr::Block(exprs)),
    }
}

/// Build expression from a pair
fn build_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    match pair.as_rule() {
        Rule::full_expr => build_full_expr(pair),
        Rule::expr_list => build_full_expr(pair),
        Rule::expr => {
            // expr is bind - get the first child which should be bind
            let mut inner = pair.into_inner();
            let bind_pair = inner.next().ok_or_else(|| ParseError::EmptyExpression)?;
            build_expr(bind_pair)
        }
        Rule::bind => build_bind(pair),
        Rule::pipe => build_pipe(pair),
        Rule::logical_or => build_logical_or(pair),
        Rule::logical_and => build_logical_and(pair),
        Rule::comparison => build_comparison(pair),
        Rule::concat => build_concat(pair),
        Rule::additive => build_additive(pair),
        Rule::multiplicative => build_multiplicative(pair),
        Rule::power => build_power(pair),
        Rule::unary => build_unary(pair),
        Rule::postfix => build_postfix(pair),
        Rule::primary => build_primary(pair),
        Rule::func_call => build_func_call(pair),
        Rule::method_chain => build_method_chain(pair),
        Rule::ident_or_lambda => {
            // ident_or_lambda can be lambda_single_param, ident, or lambda_multi_params
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::EmptyExpression)?;
            build_expr(inner)
        }
        Rule::lambda_single_param => build_lambda(pair),
        Rule::lambda_multi_params => build_lambda(pair),
        Rule::method_call => {
            let (name, args) = build_method_call(pair)?;
            Ok(Expr::Call { name, args })
        }
        Rule::list_literal => build_list_literal(pair),
        Rule::column_ref => build_column_ref(pair.as_str()),
        Rule::variable_ref => build_variable_ref(pair.as_str()),
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

fn build_bind(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let pipe_expr = inner.next().ok_or_else(|| ParseError::EmptyExpression)?;
    let mut result = build_expr(pipe_expr)?;

    // Check for 'as @name' binding
    if let Some(as_keyword) = inner.next() {
        if as_keyword.as_rule() == Rule::op_as {
            if let Some(var_name) = inner.next() {
                if var_name.as_rule() == Rule::var_name {
                    // var_name includes the @ prefix, so strip it
                    let name = var_name.as_str().trim_start_matches('@').to_string();
                    result = Expr::Bind {
                        expr: Box::new(result),
                        name,
                    };
                }
            }
        }
    }

    Ok(result)
}

fn build_pipe(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    // pipe = { logical_or ~ (op_pipe ~ pipe_func_call)* }
    // So inner should contain: [logical_or, op_pipe?, pipe_func_call?, ...]

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    // First element is always logical_or
    let first = &inner[0];
    let mut result = build_expr(first.clone())?;

    // Check if there are pipe operations (op_pipe + pipe_func_call pairs)
    let mut i = 1;
    while i < inner.len() {
        if inner[i].as_rule() == Rule::op_pipe {
            // Next should be pipe_func_call
            if i + 1 < inner.len() {
                let pipe_right = build_pipe_right(inner[i + 1].clone())?;
                result = Expr::Pipe {
                    left: Box::new(result),
                    right: Box::new(pipe_right),
                };
                i += 2;
            } else {
                break;
            }
        } else {
            // Unexpected token, skip
            i += 1;
        }
    }

    Ok(result)
}

fn build_pipe_right(pair: Pair<Rule>) -> Result<PipeRight, ParseError> {
    match pair.as_rule() {
        Rule::pipe_func_call => {
            let mut name = String::new();
            let mut args: Vec<Expr> = Vec::new();
            let mut has_placeholder = false;

            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::ident => name = inner.as_str().to_string(),
                    Rule::pipe_arg => {
                        for arg_inner in inner.into_inner() {
                            match arg_inner.as_rule() {
                                Rule::placeholder => has_placeholder = true,
                                _ => args.push(build_expr(arg_inner)?),
                            }
                        }
                    }
                    Rule::expr => args.push(build_expr(inner)?),
                    _ => {}
                }
            }

            if has_placeholder {
                Ok(PipeRight::CallWithPlaceholder { name, args })
            } else {
                Ok(PipeRight::Call { name, args })
            }
        }
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

fn build_concat(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut exprs: Vec<Expr> = Vec::new();
    let mut ops: Vec<BinaryOp> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::op_concat => ops.push(BinaryOp::Concat),
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
            // When no unary ops, unary directly contains postfix content
            Rule::postfix
            | Rule::func_call
            | Rule::primary
            | Rule::column_ref
            | Rule::variable_ref
            | Rule::list_literal
            | Rule::string
            | Rule::float
            | Rule::int
            | Rule::boolean
            | Rule::null
            | Rule::ident => {
                inner_expr = Some(build_expr(inner)?);
            }
            _ => inner_expr = Some(build_expr(inner)?),
        }
    }

    // If no inner expression found, it means unary matched but had no children
    // This happens when the rule is (op_not | op_neg)* ~ postfix and no ops matched
    // In this case, we need to handle the postfix directly
    let mut result = match inner_expr {
        Some(expr) => expr,
        None => {
            // The unary rule matched the text but didn't capture children
            // This shouldn't happen with proper grammar, but handle it gracefully
            return Err(ParseError::EmptyExpression);
        }
    };

    for op in ops.into_iter().rev() {
        result = Expr::unary(op, result);
    }

    Ok(result)
}

fn build_postfix(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::method_chain => return build_method_chain(inner),
            _ => return build_expr(inner),
        }
    }
    Err(ParseError::EmptyExpression)
}

fn build_method_chain(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();

    // First element is the primary expression (the object)
    let primary_pair = inner.next().ok_or_else(|| ParseError::EmptyExpression)?;
    let mut object = build_expr(primary_pair)?;

    // Process each method call in the chain
    for method_pair in inner {
        if method_pair.as_rule() == Rule::method_call {
            let (name, args) = build_method_call(method_pair)?;
            object = Expr::MethodCall {
                object: Box::new(object),
                name,
                args,
            };
        }
    }

    Ok(object)
}

fn build_method_call(pair: Pair<Rule>) -> Result<(String, Vec<Expr>), ParseError> {
    let mut name = String::new();
    let mut args = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => name = inner.as_str().to_string(),
            Rule::expr => args.push(build_expr(inner)?),
            _ => {}
        }
    }

    Ok((name, args))
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

fn build_list_literal(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut elements = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expr {
            elements.push(build_expr(inner)?);
        }
    }

    Ok(Expr::List(elements))
}

fn build_lambda(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner = pair.into_inner();
    let mut params = Vec::new();
    let mut body: Option<Expr> = None;

    for child in inner {
        match child.as_rule() {
            Rule::ident => {
                // Parameter: x or y
                params.push(child.as_str().to_string());
            }
            Rule::bind => {
                // Lambda body
                body = Some(build_expr(child)?);
            }
            _ => {}
        }
    }

    let body = body.ok_or_else(|| ParseError::EmptyExpression)?;

    // Transform parameter references in body to LambdaParam
    let body = transform_lambda_params(body, &params);

    Ok(Expr::Lambda {
        params,
        body: Box::new(body),
    })
}

/// Transform ColumnRef(Name(param)) to LambdaParam(param) for lambda parameters
fn transform_lambda_params(expr: Expr, params: &[String]) -> Expr {
    use ast::ColumnRef;

    match expr {
        Expr::ColumnRef(ColumnRef::Name(name)) if params.contains(&name) => {
            Expr::LambdaParam(name)
        }
        Expr::Binary { op, left, right } => Expr::Binary {
            op,
            left: Box::new(transform_lambda_params(*left, params)),
            right: Box::new(transform_lambda_params(*right, params)),
        },
        Expr::Unary { op, expr } => Expr::Unary {
            op,
            expr: Box::new(transform_lambda_params(*expr, params)),
        },
        Expr::Call { name, args } => Expr::Call {
            name,
            args: args
                .into_iter()
                .map(|a| transform_lambda_params(a, params))
                .collect(),
        },
        Expr::MethodCall { object, name, args } => Expr::MethodCall {
            object: Box::new(transform_lambda_params(*object, params)),
            name,
            args: args
                .into_iter()
                .map(|a| transform_lambda_params(a, params))
                .collect(),
        },
        Expr::Pipe { left, right } => Expr::Pipe {
            left: Box::new(transform_lambda_params(*left, params)),
            right: Box::new(transform_pipe_right(*right, params)),
        },
        Expr::Bind { expr, name } => Expr::Bind {
            expr: Box::new(transform_lambda_params(*expr, params)),
            name,
        },
        Expr::List(elements) => Expr::List(
            elements
                .into_iter()
                .map(|e| transform_lambda_params(e, params))
                .collect(),
        ),
        Expr::Block(exprs) => Expr::Block(
            exprs
                .into_iter()
                .map(|e| transform_lambda_params(e, params))
                .collect(),
        ),
        // LambdaParam, ColumnRef(Index), Int, Float, String, Bool, Null, Variable, Lambda remain unchanged
        other => other,
    }
}

/// Transform parameter references in pipe right-hand side
fn transform_pipe_right(pipe_right: PipeRight, params: &[String]) -> PipeRight {
    match pipe_right {
        PipeRight::Call { name, args } => PipeRight::Call {
            name,
            args: args
                .into_iter()
                .map(|a| transform_lambda_params(a, params))
                .collect(),
        },
        PipeRight::CallWithPlaceholder { name, args } => {
            PipeRight::CallWithPlaceholder {
                name,
                args: args
                    .into_iter()
                    .map(|a| transform_lambda_params(a, params))
                    .collect(),
            }
        }
    }
}

fn build_primary(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    for inner in pair.into_inner() {
        return build_expr(inner);
    }
    Err(ParseError::EmptyExpression)
}

fn build_column_ref(s: &str) -> Result<Expr, ParseError> {
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

fn build_variable_ref(s: &str) -> Result<Expr, ParseError> {
    let name = extract_variable_name(s);
    Ok(Expr::Variable(name))
}

fn extract_variable_name(s: &str) -> String {
    s[1..].to_string() // Remove '@' prefix
}

fn build_string(s: &str) -> Result<Expr, ParseError> {
    let inner = &s[1..s.len() - 1];
    Ok(Expr::String(inner.to_string()))
}

fn fold_left(exprs: Vec<Expr>, ops: Vec<BinaryOp>) -> Result<Expr, ParseError> {
    if exprs.is_empty() {
        return Err(ParseError::EmptyExpression);
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
    fn test_parse_variable_ref() {
        // @name is parsed as ColumnRef::Name, not Variable
        // Variable resolution happens at runtime based on context
        let expr = parse("@total").unwrap();
        assert!(matches!(expr, Expr::ColumnRef(ColumnRef::Name(s)) if s == "total"));
    }

    #[test]
    fn test_parse_list_literal() {
        let expr = parse("[1, 2, 3]").unwrap();
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 3);
                assert!(matches!(elements[0], Expr::Int(1)));
                assert!(matches!(elements[1], Expr::Int(2)));
                assert!(matches!(elements[2], Expr::Int(3)));
            }
            _ => panic!("Expected List expression"),
        }
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
        let expr = parse("@1 > 0 && @2 < 100").unwrap();
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
    fn test_parse_method_call() {
        // @name.trim() should parse as MethodCall { object: @name, name: "trim", args: [] }
        let expr = parse("@name.trim()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert!(
                    matches!(*object, Expr::ColumnRef(ColumnRef::Name(s)) if s == "name")
                );
                assert_eq!(name, "trim");
                assert!(args.is_empty());
            }
            _ => panic!("Expected MethodCall expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_method_call_with_args() {
        // @name.substr(0, 5) should parse as MethodCall { object: @name, name: "substr", args: [0, 5] }
        let expr = parse("@name.substr(0, 5)").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert!(
                    matches!(*object, Expr::ColumnRef(ColumnRef::Name(s)) if s == "name")
                );
                assert_eq!(name, "substr");
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0], Expr::Int(0)));
                assert!(matches!(args[1], Expr::Int(5)));
            }
            _ => panic!("Expected MethodCall expression with args"),
        }
    }

    #[test]
    fn test_parse_method_chain() {
        // @name.trim().upper() should parse as MethodCall { object: MethodCall { object: @name, name: "trim" }, name: "upper" }
        let expr = parse("@name.trim().upper()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert_eq!(name, "upper");
                assert!(args.is_empty());
                // Check inner method call
                match *object {
                    Expr::MethodCall {
                        object: inner_obj,
                        name: inner_name,
                        args: inner_args,
                    } => {
                        assert_eq!(inner_name, "trim");
                        assert!(inner_args.is_empty());
                        assert!(
                            matches!(*inner_obj, Expr::ColumnRef(ColumnRef::Name(s)) if s == "name")
                        );
                    }
                    _ => panic!("Expected nested MethodCall for trim"),
                }
            }
            _ => panic!("Expected MethodCall expression for method chain"),
        }
    }

    #[test]
    fn test_parse_lambda() {
        // x => x + 1
        let expr = parse("x => x + 1").unwrap();
        println!("Parsed expr: {:?}", expr);
        match expr {
            Expr::Lambda { params, body } => {
                assert_eq!(params, vec!["x"]);
                match *body {
                    Expr::Binary {
                        op: BinaryOp::Add,
                        left,
                        right,
                    } => {
                        assert!(matches!(*left, Expr::LambdaParam(s) if s == "x"));
                        assert!(matches!(*right, Expr::Int(1)));
                    }
                    _ => {
                        panic!("Expected Add expression in lambda body, got {:?}", body)
                    }
                }
            }
            _ => panic!("Expected Lambda expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_lambda_multi_params() {
        // (x, y) => x + y
        let expr = parse("(x, y) => x + y").unwrap();
        match expr {
            Expr::Lambda { params, body } => {
                assert_eq!(params, vec!["x", "y"]);
                match *body {
                    Expr::Binary {
                        op: BinaryOp::Add,
                        left,
                        right,
                    } => {
                        assert!(matches!(*left, Expr::LambdaParam(s) if s == "x"));
                        assert!(matches!(*right, Expr::LambdaParam(s) if s == "y"));
                    }
                    _ => panic!("Expected Add expression in lambda body"),
                }
            }
            _ => panic!("Expected Lambda expression"),
        }
    }

    #[test]
    fn test_parse_lambda_no_params() {
        // () => 42
        let expr = parse("() => 42").unwrap();
        match expr {
            Expr::Lambda { params, body } => {
                assert!(params.is_empty());
                assert!(matches!(*body, Expr::Int(42)));
            }
            _ => panic!("Expected Lambda expression"),
        }
    }

    #[test]
    fn test_parse_errors() {
        assert!(parse("@").is_err());
        assert!(parse("@0").is_err());
        assert!(parse("").is_err());
    }
}
