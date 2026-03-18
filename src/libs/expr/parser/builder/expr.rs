use crate::libs::expr::parser::ast::{Expr, PipeRight};
use crate::libs::expr::parser::ParseError;
use crate::libs::expr::parser::Rule;
use pest::iterators::Pair;

pub fn build_full_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut exprs = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr_list => {
                for expr_pair in inner.into_inner() {
                    match expr_pair.as_rule() {
                        Rule::expr => {
                            exprs.push(super::build_expr(expr_pair)?);
                        }
                        _ => {}
                    }
                }
            }
            Rule::expr => {
                exprs.push(super::build_expr(inner)?);
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

pub fn build_bind(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut result = super::build_expr(inner[0].clone())?;

    let mut i = 1;
    if i < inner.len() && inner[i].as_rule() == Rule::op_as {
        i += 1;

        if i < inner.len() && inner[i].as_rule() == Rule::var_name {
            let name = inner[i].as_str().trim_start_matches('@').to_string();
            i += 1;

            result = Expr::Bind {
                expr: Box::new(result),
                name,
            };

            while i < inner.len() {
                if inner[i].as_rule() == Rule::op_pipe {
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

pub fn build_pipe(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let mut result = super::build_expr(inner[0].clone())?;

    let mut i = 1;
    while i < inner.len() {
        if inner[i].as_rule() == Rule::op_pipe {
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

pub fn build_pipe_right(pair: Pair<Rule>) -> Result<PipeRight, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if inner.is_empty() {
        return Err(ParseError::EmptyExpression);
    }

    let name = inner[0].as_str().to_string();
    let mut args = Vec::new();

    let mut i = 1;
    while i < inner.len() {
        if inner[i].as_rule() == Rule::pipe_arg {
            let arg_inner: Vec<Pair<Rule>> = inner[i].clone().into_inner().collect();
            if !arg_inner.is_empty() {
                if arg_inner[0].as_rule() == Rule::placeholder {
                    args.push(Expr::Underscore);
                } else {
                    args.push(super::build_expr(arg_inner[0].clone())?);
                }
            }
        }
        i += 1;
    }

    if args.iter().any(|arg| matches!(arg, Expr::Underscore)) {
        Ok(PipeRight::CallWithPlaceholder { name, args })
    } else {
        Ok(PipeRight::Call { name, args })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::expr::parser::parse;

    #[test]
    fn test_build_full_expr_single() {
        let expr = parse("1 + 2").unwrap();
        match expr {
            Expr::Binary { .. } => {}
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_build_full_expr_block() {
        let expr = parse("1 + 2; 3 + 4").unwrap();
        match expr {
            Expr::Block(exprs) => {
                assert_eq!(exprs.len(), 2);
            }
            _ => panic!("Expected block expression"),
        }
    }

    #[test]
    fn test_build_bind_with_as() {
        let expr = parse("1 + 2 as @result").unwrap();
        match expr {
            Expr::Bind { name, .. } => {
                assert_eq!(name, "result");
            }
            _ => panic!("Expected bind expression"),
        }
    }

    #[test]
    fn test_build_pipe_simple() {
        let expr = parse("[1,2,3] | len()").unwrap();
        match expr {
            Expr::Pipe { .. } => {}
            _ => panic!("Expected pipe expression"),
        }
    }

    #[test]
    fn test_build_pipe_with_placeholder() {
        let expr = parse("[1,2,3] | map(_, x => x * 2)").unwrap();
        match expr {
            Expr::Pipe { right, .. } => match *right {
                PipeRight::CallWithPlaceholder { .. } => {}
                _ => panic!("Expected CallWithPlaceholder"),
            },
            _ => panic!("Expected pipe expression"),
        }
    }
}
