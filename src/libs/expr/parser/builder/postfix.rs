use crate::libs::expr::parser::ast::Expr;
use crate::libs::expr::parser::ParseError;
use crate::libs::expr::parser::Rule;
use pest::iterators::Pair;

pub fn build_postfix(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();

    // First element can be func_call or primary
    let first_pair = inner.next().ok_or(ParseError::EmptyExpression)?;

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

    #[test]
    fn test_method_chain_three_levels() {
        // Test three-level method chain: "hello".upper().trim().len()
        let expr = parse("\"hello\".upper().trim().len()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert_eq!(name, "len");
                assert!(args.is_empty());
                // Second level: "hello".upper().trim()
                match object.as_ref() {
                    Expr::MethodCall {
                        object: inner_obj,
                        name: inner_name,
                        args: inner_args,
                    } => {
                        assert_eq!(inner_name, "trim");
                        assert!(inner_args.is_empty());
                        // First level: "hello".upper()
                        match inner_obj.as_ref() {
                            Expr::MethodCall {
                                object: innermost_obj,
                                name: innermost_name,
                                args: innermost_args,
                            } => {
                                assert_eq!(innermost_name, "upper");
                                assert!(innermost_args.is_empty());
                                assert!(
                                    matches!(innermost_obj.as_ref(), Expr::String(s) if s == "hello")
                                );
                            }
                            _ => panic!("Expected innermost MethodCall"),
                        }
                    }
                    _ => panic!("Expected nested MethodCall at level 2"),
                }
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_method_call_with_complex_args() {
        // Method call with expression arguments
        let expr = parse("\"hello\".replace(\"l\", \"x\")").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert!(matches!(object.as_ref(), Expr::String(s) if s == "hello"));
                assert_eq!(name, "replace");
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0], Expr::String(ref s) if s == "l"));
                assert!(matches!(args[1], Expr::String(ref s) if s == "x"));
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_method_call_with_column_arg() {
        // Method call with column reference as argument
        let expr = parse("\"hello\".concat(@name)").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert!(matches!(object.as_ref(), Expr::String(s) if s == "hello"));
                assert_eq!(name, "concat");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expr::ColumnRef(_)));
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_func_call_followed_by_method_chain() {
        // Function call followed by method chain: now().to_string().upper()
        let expr = parse("now().to_string().upper()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert_eq!(name, "upper");
                assert!(args.is_empty());
                match object.as_ref() {
                    Expr::MethodCall {
                        object: inner_obj,
                        name: inner_name,
                        args: inner_args,
                    } => {
                        assert_eq!(inner_name, "to_string");
                        assert!(inner_args.is_empty());
                        // Innermost should be the function call
                        assert!(matches!(inner_obj.as_ref(), Expr::Call { .. }));
                    }
                    _ => panic!("Expected nested MethodCall"),
                }
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_method_call_on_list_literal() {
        // Method call on list literal: [1, 2, 3].len()
        let expr = parse("[1, 2, 3].len()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert!(matches!(object.as_ref(), Expr::List(_)));
                assert_eq!(name, "len");
                assert!(args.is_empty());
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_method_call_on_binary_expr() {
        // Method call on parenthesized binary expression: (1 + 2).to_string()
        let expr = parse("(1 + 2).to_string()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert_eq!(name, "to_string");
                assert!(args.is_empty());
                assert!(matches!(object.as_ref(), Expr::Binary { .. }));
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_func_call_with_multiple_expr_args() {
        // Function with multiple expression arguments
        let expr = parse("substr(@name, 1 + 2, 3 * 4)").unwrap();
        match expr {
            Expr::Call { name, args } => {
                assert_eq!(name, "substr");
                assert_eq!(args.len(), 3);
                assert!(matches!(args[0], Expr::ColumnRef(_)));
                assert!(matches!(args[1], Expr::Binary { .. }));
                assert!(matches!(args[2], Expr::Binary { .. }));
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_method_call_chained_on_column() {
        // Method chain starting from column: @name.trim().upper()
        let expr = parse("@name.trim().upper()").unwrap();
        match expr {
            Expr::MethodCall { object, name, args } => {
                assert_eq!(name, "upper");
                assert!(args.is_empty());
                match object.as_ref() {
                    Expr::MethodCall {
                        object: inner_obj,
                        name: inner_name,
                        args: inner_args,
                    } => {
                        assert_eq!(inner_name, "trim");
                        assert!(inner_args.is_empty());
                        assert!(matches!(inner_obj.as_ref(), Expr::ColumnRef(_)));
                    }
                    _ => panic!("Expected nested MethodCall"),
                }
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }
}
