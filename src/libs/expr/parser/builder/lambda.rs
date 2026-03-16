use super::{Expr, ParseError, PipeRight};
use pest::iterators::Pair;

pub fn build_lambda(pair: Pair<super::super::Rule>) -> Result<Expr, ParseError> {
    let inner = pair.into_inner();
    let mut params = Vec::new();
    let mut body: Option<Expr> = None;

    for child in inner {
        match child.as_rule() {
            super::super::Rule::ident => {
                params.push(child.as_str().to_string());
            }
            super::super::Rule::bind => {
                body = Some(super::build_expr(child)?);
            }
            _ => {}
        }
    }

    let body = body.ok_or_else(|| ParseError::EmptyExpression)?;
    let body = transform_lambda_params(body, &params);

    Ok(Expr::Lambda {
        params,
        body: Box::new(body),
    })
}

fn transform_lambda_params(expr: Expr, params: &[String]) -> Expr {
    use super::ColumnRef;

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
        other => other,
    }
}

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
