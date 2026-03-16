use crate::libs::expr::parser::ast::Expr;
use crate::libs::expr::parser::ParseError;
use crate::libs::expr::parser::Rule;
use pest::iterators::Pair;

pub fn build_postfix(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();

    // First element can be func_call or primary
    let first_pair = inner.next().ok_or_else(|| ParseError::EmptyExpression)?;

    // If it's a func_call, return it directly (no method chain after standalone func_call)
    if first_pair.as_rule() == Rule::func_call {
        return build_func_call(first_pair);
    }

    // Otherwise, it's a primary, build it and process method chain
    let mut object = super::build_expr(first_pair)?;

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

pub fn build_method_call(pair: Pair<Rule>) -> Result<(String, Vec<Expr>), ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let name = inner[0].as_str().to_string();
    let mut args = Vec::new();

    for i in 1..inner.len() {
        if inner[i].as_rule() == Rule::expr {
            args.push(super::build_expr(inner[i].clone())?);
        }
    }

    Ok((name, args))
}

pub fn build_func_call(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let name = inner[0].as_str().to_string();
    let mut args = Vec::new();

    for i in 1..inner.len() {
        if inner[i].as_rule() == Rule::expr {
            args.push(super::build_expr(inner[i].clone())?);
        }
    }

    Ok(Expr::Call { name, args })
}
