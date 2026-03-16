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
        let expr = parse("3.14").unwrap();
        assert!(matches!(expr, Expr::Float(n) if (n - 3.14).abs() < 0.001));
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
