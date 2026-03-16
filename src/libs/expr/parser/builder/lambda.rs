use crate::libs::expr::parser::ast::{ColumnRef, Expr, PipeRight};
use crate::libs::expr::parser::ParseError;
use crate::libs::expr::parser::Rule;
use pest::iterators::Pair;

pub fn build_lambda(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner = pair.into_inner();
    let mut params = Vec::new();
    let mut body: Option<Expr> = None;

    for child in inner {
        match child.as_rule() {
            Rule::ident => {
                params.push(child.as_str().to_string());
            }
            Rule::bind => {
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

#[cfg(test)]
mod tests {
    use crate::libs::expr::parser::ast::{BinaryOp, Expr};
    use crate::libs::expr::parser::parse;

    #[test]
    fn test_parse_lambda() {
        // x => x + 1
        let expr = parse("x => x + 1").unwrap();
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
                    _ => {
                        panic!("Expected Add expression in lambda body, got {:?}", body)
                    }
                }
            }
            _ => panic!("Expected Lambda expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_lambda_with_call() {
        // x => abs(x)
        let expr = parse("x => abs(x)").unwrap();
        match expr {
            Expr::Lambda { params, body } => {
                assert_eq!(params, vec!["x"]);
                match *body {
                    Expr::Call { name, args } => {
                        assert_eq!(name, "abs");
                        assert_eq!(args.len(), 1);
                        assert!(matches!(&args[0], Expr::LambdaParam(s) if s == "x"));
                    }
                    _ => {
                        panic!("Expected Call expression in lambda body, got {:?}", body)
                    }
                }
            }
            _ => panic!("Expected Lambda expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_lambda_with_pipe() {
        // x => x | abs()
        let expr = parse("x => x | abs()").unwrap();
        match expr {
            Expr::Lambda { params, body } => {
                assert_eq!(params, vec!["x"]);
                match *body {
                    Expr::Pipe { left, right } => {
                        assert!(matches!(*left, Expr::LambdaParam(s) if s == "x"));
                        match *right {
                            super::PipeRight::Call { name, args } => {
                                assert_eq!(name, "abs");
                                assert!(args.is_empty());
                            }
                            _ => panic!("Expected Call in pipe right, got {:?}", right),
                        }
                    }
                    _ => {
                        panic!("Expected Pipe expression in lambda body, got {:?}", body)
                    }
                }
            }
            _ => panic!("Expected Lambda expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_lambda_with_pipe_placeholder() {
        // x => x | pow(_, 2)
        let expr = parse("x => x | pow(_, 2)").unwrap();
        match expr {
            Expr::Lambda { params, body } => {
                assert_eq!(params, vec!["x"]);
                match *body {
                    Expr::Pipe { left, right } => {
                        assert!(matches!(*left, Expr::LambdaParam(s) if s == "x"));
                        match *right {
                            super::PipeRight::CallWithPlaceholder { name, args } => {
                                assert_eq!(name, "pow");
                                assert_eq!(args.len(), 2);
                                assert!(
                                    matches!(&args[0], Expr::LambdaParam(s) if s == "_")
                                );
                                assert!(matches!(&args[1], Expr::Int(2)));
                            }
                            _ => panic!(
                                "Expected CallWithPlaceholder in pipe right, got {:?}",
                                right
                            ),
                        }
                    }
                    _ => {
                        panic!("Expected Pipe expression in lambda body, got {:?}", body)
                    }
                }
            }
            _ => panic!("Expected Lambda expression, got {:?}", expr),
        }
    }
}
