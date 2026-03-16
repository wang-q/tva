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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::expr::parser::parse;

    #[test]
    fn test_func_call_no_args() {
        let expr = parse("now()").unwrap();
        match expr {
            Expr::Call { name, args } => {
                assert_eq!(name, "now");
                assert!(args.is_empty());
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_func_call_single_arg() {
        let expr = parse("abs(-5)").unwrap();
        match expr {
            Expr::Call { name, args } => {
                assert_eq!(name, "abs");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expr::Unary { .. }));
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_func_call_multiple_args() {
        let expr = parse("substr(@name, 0, 5)").unwrap();
        match expr {
            Expr::Call { name, args } => {
                assert_eq!(name, "substr");
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_method_call_simple() {
        let expr = parse("\"hello\".upper()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert!(matches!(object.as_ref(), Expr::String(s) if s == "hello"));
                assert_eq!(name, "upper");
                assert!(args.is_empty());
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_method_call_with_args() {
        let expr = parse("\"hello\".replace(\"l\", \"x\")").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert!(matches!(object.as_ref(), Expr::String(s) if s == "hello"));
                assert_eq!(name, "replace");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_method_chain() {
        let expr = parse("\"hello\".upper().len()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert_eq!(name, "len");
                assert!(args.is_empty());
                // object should be "hello".upper()
                match object.as_ref() {
                    Expr::MethodCall {
                        object: inner_obj,
                        name: inner_name,
                        args: inner_args,
                    } => {
                        assert!(
                            matches!(inner_obj.as_ref(), Expr::String(s) if s == "hello")
                        );
                        assert_eq!(inner_name, "upper");
                        assert!(inner_args.is_empty());
                    }
                    _ => panic!("Expected nested MethodCall expression"),
                }
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_method_call_on_column() {
        let expr = parse("@name.upper()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert!(matches!(object.as_ref(), Expr::ColumnRef(_)));
                assert_eq!(name, "upper");
                assert!(args.is_empty());
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_method_call_on_func_result() {
        let expr = parse("abs(@x).to_string()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert_eq!(name, "to_string");
                assert!(args.is_empty());
                // object should be abs(@x)
                assert!(matches!(object.as_ref(), Expr::Call { .. }));
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_nested_func_calls() {
        let expr = parse("abs(min(1, 2))").unwrap();
        match expr {
            Expr::Call { name, args } => {
                assert_eq!(name, "abs");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expr::Call { .. }));
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_func_call_with_expr_arg() {
        let expr = parse("abs(1 + 2)").unwrap();
        match expr {
            Expr::Call { name, args } => {
                assert_eq!(name, "abs");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expr::Binary { .. }));
            }
            _ => panic!("Expected Call expression"),
        }
    }
}
