use super::ast::{BinaryOp, ColumnRef, Expr, PipeRight, UnaryOp};
use super::ParseError;
use pest::iterators::Pair;

mod lambda;
mod literal;
mod primary;

pub use lambda::*;
pub use literal::*;
pub use primary::*;

/// Build full expression (handles multiple expressions separated by semicolons)
pub fn build_full_expr(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let mut exprs = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            super::Rule::expr_list => {
                for expr_pair in inner.into_inner() {
                    match expr_pair.as_rule() {
                        super::Rule::expr => {
                            exprs.push(build_expr(expr_pair)?);
                        }
                        _ => {}
                    }
                }
            }
            super::Rule::expr => {
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
pub fn build_expr(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    match pair.as_rule() {
        super::Rule::full_expr => build_full_expr(pair),
        super::Rule::expr_list => build_full_expr(pair),
        super::Rule::expr => {
            let mut inner = pair.into_inner();
            let bind_pair = inner.next().ok_or_else(|| ParseError::EmptyExpression)?;
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
                .ok_or_else(|| ParseError::EmptyExpression)?;
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
            Ok(Expr::ColumnRef(ColumnRef::Name(pair.as_str().to_string())))
        }
        _ => Err(ParseError::UnexpectedRule(pair.as_rule())),
    }
}

fn build_bind(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut result = build_expr(inner[0].clone())?;

    let mut i = 1;
    if i < inner.len() && inner[i].as_rule() == super::Rule::op_as {
        i += 1;

        if i < inner.len() && inner[i].as_rule() == super::Rule::var_name {
            let name = inner[i].as_str().trim_start_matches('@').to_string();
            i += 1;

            result = Expr::Bind {
                expr: Box::new(result),
                name,
            };

            while i < inner.len() {
                if inner[i].as_rule() == super::Rule::op_pipe {
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
                    i += 1;
                }
            }
        }
    }

    Ok(result)
}

fn build_pipe(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut result = build_expr(inner[0].clone())?;

    let mut i = 1;
    while i < inner.len() {
        if inner[i].as_rule() == super::Rule::op_pipe {
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
            i += 1;
        }
    }

    Ok(result)
}

fn build_pipe_right(pair: Pair<super::Rule>) -> Result<PipeRight, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let name = inner[0].as_str().to_string();
    let mut args = Vec::new();

    let mut i = 1;
    while i < inner.len() {
        if inner[i].as_rule() == super::Rule::pipe_arg {
            let arg_inner: Vec<Pair<super::Rule>> =
                inner[i].clone().into_inner().collect();
            if !arg_inner.is_empty() {
                if arg_inner[0].as_rule() == super::Rule::placeholder {
                    args.push(Expr::LambdaParam("_".to_string()));
                } else {
                    args.push(build_expr(arg_inner[0].clone())?);
                }
            }
        }
        i += 1;
    }

    if args.iter().any(|arg| matches!(arg, Expr::LambdaParam(_))) {
        Ok(PipeRight::CallWithPlaceholder { name, args })
    } else {
        Ok(PipeRight::Call { name, args })
    }
}

fn build_logical_or(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(build_expr(item.clone())?);
        } else if item.as_rule() == super::Rule::op_or {
            ops.push(BinaryOp::Or);
        }
    }

    fold_left(exprs, ops)
}

fn build_logical_and(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(build_expr(item.clone())?);
        } else if item.as_rule() == super::Rule::op_and {
            ops.push(BinaryOp::And);
        }
    }

    fold_left(exprs, ops)
}

fn build_comparison(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(build_expr(item.clone())?);
        } else {
            let op = match item.as_rule() {
                super::Rule::op_eq => BinaryOp::Eq,
                super::Rule::op_ne => BinaryOp::Ne,
                super::Rule::op_lt => BinaryOp::Lt,
                super::Rule::op_le => BinaryOp::Le,
                super::Rule::op_gt => BinaryOp::Gt,
                super::Rule::op_ge => BinaryOp::Ge,
                super::Rule::op_str_eq => BinaryOp::StrEq,
                super::Rule::op_str_ne => BinaryOp::StrNe,
                super::Rule::op_str_lt => BinaryOp::StrLt,
                super::Rule::op_str_le => BinaryOp::StrLe,
                super::Rule::op_str_gt => BinaryOp::StrGt,
                super::Rule::op_str_ge => BinaryOp::StrGe,
                _ => return Err(ParseError::UnexpectedRule(item.as_rule())),
            };
            ops.push(op);
        }
    }

    fold_left(exprs, ops)
}

fn build_concat(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();

    for item in inner {
        if item.as_rule() != super::Rule::op_concat {
            exprs.push(build_expr(item)?);
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

fn build_additive(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(build_expr(item.clone())?);
        } else {
            let op = match item.as_rule() {
                super::Rule::op_add => BinaryOp::Add,
                super::Rule::op_sub => BinaryOp::Sub,
                _ => return Err(ParseError::UnexpectedRule(item.as_rule())),
            };
            ops.push(op);
        }
    }

    fold_left(exprs, ops)
}

fn build_multiplicative(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(build_expr(item.clone())?);
        } else {
            let op = match item.as_rule() {
                super::Rule::op_mul => BinaryOp::Mul,
                super::Rule::op_div => BinaryOp::Div,
                super::Rule::op_mod => BinaryOp::Mod,
                _ => return Err(ParseError::UnexpectedRule(item.as_rule())),
            };
            ops.push(op);
        }
    }

    fold_left(exprs, ops)
}

fn build_power(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut exprs = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in inner.iter().enumerate() {
        if i % 2 == 0 {
            exprs.push(build_expr(item.clone())?);
        } else if item.as_rule() == super::Rule::op_pow {
            ops.push(BinaryOp::Pow);
        }
    }

    fold_left(exprs, ops)
}

fn build_unary(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let mut ops: Vec<UnaryOp> = Vec::new();
    let mut inner_pairs: Vec<Pair<super::Rule>> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            super::Rule::op_not => ops.push(UnaryOp::Not),
            super::Rule::op_neg => ops.push(UnaryOp::Neg),
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
            super::Rule::func_call => build_func_call(first.clone())?,
            super::Rule::postfix => build_postfix(first.clone())?,
            super::Rule::primary => build_primary(first.clone())?,
            _ => build_expr(first.clone())?,
        };

        // Remaining pairs are method calls
        let mut obj = base;
        for method_pair in inner_pairs.iter().skip(1) {
            if method_pair.as_rule() == super::Rule::method_call {
                let (name, args) = build_method_call(method_pair.clone())?;
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

fn build_postfix(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();

    // First element can be func_call or primary
    let first_pair = inner.next().ok_or_else(|| ParseError::EmptyExpression)?;

    // If it's a func_call, return it directly (no method chain after standalone func_call)
    if first_pair.as_rule() == super::Rule::func_call {
        return build_func_call(first_pair);
    }

    // Otherwise, it's a primary, build it and process method chain
    let mut object = build_expr(first_pair)?;

    // Process each method call in the chain
    for method_pair in inner {
        if method_pair.as_rule() == super::Rule::method_call {
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

fn build_method_call(
    pair: Pair<super::Rule>,
) -> Result<(String, Vec<Expr>), ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let name = inner[0].as_str().to_string();
    let mut args = Vec::new();

    for i in 1..inner.len() {
        if inner[i].as_rule() == super::Rule::expr {
            args.push(build_expr(inner[i].clone())?);
        }
    }

    Ok((name, args))
}

fn build_func_call(pair: Pair<super::Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<super::Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let name = inner[0].as_str().to_string();
    let mut args = Vec::new();

    for i in 1..inner.len() {
        if inner[i].as_rule() == super::Rule::expr {
            args.push(build_expr(inner[i].clone())?);
        }
    }

    Ok(Expr::Call { name, args })
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
        result = Expr::Binary {
            op: *op,
            left: Box::new(result),
            right: Box::new(exprs[i + 1].clone()),
        };
    }
    Ok(result)
}
