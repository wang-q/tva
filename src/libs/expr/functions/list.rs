use crate::libs::expr::runtime::value::{LambdaValue, Value};
use crate::libs::expr::runtime::{eval, EvalContext, EvalError};

pub fn join(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let sep = args[1].as_string();
            let parts: Vec<String> = list.iter().map(|v| v.to_string()).collect();
            Ok(Value::String(parts.join(&sep)))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "join: first argument must be a list".to_string(),
        )),
    }
}

pub fn first(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            if let Some(first) = list.first() {
                Ok(first.clone())
            } else {
                Ok(Value::Null)
            }
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "first: argument must be a list".to_string(),
        )),
    }
}

pub fn last(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            if let Some(last) = list.last() {
                Ok(last.clone())
            } else {
                Ok(Value::Null)
            }
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "last: argument must be a list".to_string(),
        )),
    }
}

pub fn reverse(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let mut reversed = list.clone();
            reversed.reverse();
            Ok(Value::List(reversed))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "reverse: argument must be a list".to_string(),
        )),
    }
}

pub fn nth(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let n = match &args[1] {
                Value::Int(i) => *i as usize,
                Value::Float(f) => f.round() as usize,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "nth: index must be a number, got {}",
                        v.type_name()
                    )))
                }
            };
            // Support negative indexing like Python
            let idx = if n >= list.len() {
                return Ok(Value::Null);
            } else {
                n
            };
            Ok(list.get(idx).cloned().unwrap_or(Value::Null))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "nth: first argument must be a list".to_string(),
        )),
    }
}

/// Replace the nth element of a list with a new value
/// Returns a new list, original list is unchanged
pub fn replace_nth(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let n = match &args[1] {
                Value::Int(i) => *i as usize,
                Value::Float(f) => f.round() as usize,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "replace_nth: index must be a number, got {}",
                        v.type_name()
                    )))
                }
            };

            if n >= list.len() {
                return Err(EvalError::TypeError(format!(
                    "replace_nth: index {} out of bounds for list of length {}",
                    n,
                    list.len()
                )));
            }

            let mut new_list = list.clone();
            new_list[n] = args[2].clone();
            Ok(Value::List(new_list))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "replace_nth: first argument must be a list".to_string(),
        )),
    }
}

pub fn sort(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let mut sorted = list.clone();
            sorted.sort_by(|a, b| {
                // Try numeric comparison first
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => a.cmp(b),
                    (Value::Float(a), Value::Float(b)) => {
                        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::Int(a), Value::Float(b)) => (*a as f64)
                        .partial_cmp(b)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    (Value::Float(a), Value::Int(b)) => a
                        .partial_cmp(&(*b as f64))
                        .unwrap_or(std::cmp::Ordering::Equal),
                    // Fall back to string comparison
                    _ => a.to_string().cmp(&b.to_string()),
                }
            });
            Ok(Value::List(sorted))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "sort: argument must be a list".to_string(),
        )),
    }
}

pub fn unique(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let mut seen = std::collections::HashSet::new();
            let mut result = Vec::new();
            for item in list {
                // Use string representation for comparison
                let key = item.to_string();
                if seen.insert(key) {
                    result.push(item.clone());
                }
            }
            Ok(Value::List(result))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "unique: argument must be a list".to_string(),
        )),
    }
}

pub fn slice(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let start = match &args[1] {
                Value::Int(i) => *i as usize,
                Value::Float(f) => f.round() as usize,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "slice: start must be a number, got {}",
                        v.type_name()
                    )))
                }
            };
            let end = if args.len() > 2 {
                match &args[2] {
                    Value::Int(i) => Some(*i as usize),
                    Value::Float(f) => Some(f.round() as usize),
                    v => {
                        return Err(EvalError::TypeError(format!(
                            "slice: end must be a number, got {}",
                            v.type_name()
                        )))
                    }
                }
            } else {
                None
            };

            let start = start.min(list.len());
            let end = end.map(|e| e.min(list.len())).unwrap_or(list.len());
            let end = if end < start { start } else { end };

            Ok(Value::List(list[start..end].to_vec()))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "slice: first argument must be a list".to_string(),
        )),
    }
}

pub fn reduce(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let lambda = match &args[2] {
                Value::Lambda(l) => l,
                _ => {
                    return Err(EvalError::TypeError(
                        "reduce: third argument must be a lambda".to_string(),
                    ))
                }
            };

            if lambda.params.len() < 2 {
                return Err(EvalError::TypeError(
                    "reduce: lambda must have at least 2 parameters (acc, item)"
                        .to_string(),
                ));
            }

            let mut result = args[1].clone();

            // Create a context with captured variables for lambda evaluation
            let dummy_row: Vec<String> = vec![];
            let mut ctx = EvalContext::new(&dummy_row);
            ctx.variables = lambda.captured_vars.clone();

            for item in list.iter() {
                // Bind accumulator to first parameter, item to second parameter
                ctx.set_lambda_param(lambda.params[0].clone(), result);
                ctx.set_lambda_param(lambda.params[1].clone(), item.clone());

                // Evaluate the lambda body
                result = eval(&lambda.body, &mut ctx)?;

                // Clear lambda parameters after evaluation
                ctx.clear_lambda_params();
            }

            Ok(result)
        }
        Value::Null => Ok(args[1].clone()),
        _ => Err(EvalError::TypeError(
            "reduce: first argument must be a list".to_string(),
        )),
    }
}

/// Apply a lambda function to each element of a list
fn apply_lambda(lambda: &LambdaValue, item: &Value) -> Result<Value, EvalError> {
    // Set lambda parameters
    if lambda.params.is_empty() {
        return Err(EvalError::TypeError("lambda has no parameters".to_string()));
    }

    // Create a context with captured variables for lambda evaluation
    let dummy_row: Vec<String> = vec![];
    let mut ctx = EvalContext::new(&dummy_row);
    ctx.variables = lambda.captured_vars.clone();

    // Bind the first parameter to the item
    ctx.set_lambda_param(lambda.params[0].clone(), item.clone());

    // Evaluate the lambda body
    eval(&lambda.body, &mut ctx)
}

pub fn map(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let lambda = match &args[1] {
                Value::Lambda(l) => l,
                _ => {
                    return Err(EvalError::TypeError(
                        "map: second argument must be a lambda".to_string(),
                    ))
                }
            };

            let mut result = Vec::with_capacity(list.len());
            for item in list {
                let mapped = apply_lambda(lambda, item)?;
                result.push(mapped);
            }

            Ok(Value::List(result))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "map: first argument must be a list".to_string(),
        )),
    }
}

pub fn filter(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let lambda = match &args[1] {
                Value::Lambda(l) => l,
                _ => {
                    return Err(EvalError::TypeError(
                        "filter: second argument must be a lambda".to_string(),
                    ))
                }
            };

            let mut result = Vec::new();
            for item in list {
                let keep = apply_lambda(lambda, item)?;
                if keep.as_bool() {
                    result.push(item.clone());
                }
            }

            Ok(Value::List(result))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "filter: first argument must be a list".to_string(),
        )),
    }
}

/// Sort a list by a key extracted by a lambda function
/// Each element is transformed by the lambda to produce a sort key
/// Keys are cached to avoid calling lambda multiple times per element
pub fn sort_by(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let lambda = match &args[1] {
                Value::Lambda(l) => l,
                _ => {
                    return Err(EvalError::TypeError(
                        "sort_by: second argument must be a lambda".to_string(),
                    ))
                }
            };

            // Pre-compute keys for all elements (cache to avoid multiple lambda calls)
            let mut keyed: Vec<(Value, Value)> = Vec::with_capacity(list.len());
            for item in list.iter() {
                let key = apply_lambda(lambda, item)?;
                keyed.push((key, item.clone()));
            }

            // Sort by key using Value::compare
            keyed.sort_by(|(a, _), (b, _)| {
                a.compare(b).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Extract sorted values
            let result: Vec<Value> = keyed.into_iter().map(|(_, v)| v).collect();
            Ok(Value::List(result))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "sort_by: first argument must be a list".to_string(),
        )),
    }
}

/// Generate a range of numbers
/// range(upto) -> [0, 1, ..., upto-1]
/// range(from, upto) -> [from, from+1, ..., upto-1]
/// range(from, upto, by) -> [from, from+by, ...] while < upto (or > upto if by is negative)
pub fn range(args: &[Value]) -> Result<Value, EvalError> {
    let (from, upto, by) = match args.len() {
        1 => {
            // range(upto): from=0, upto=arg, by=1
            let upto = match &args[0] {
                Value::Int(i) => *i,
                Value::Float(f) => f.round() as i64,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "range: argument must be a number, got {}",
                        v.type_name()
                    )))
                }
            };
            (0i64, upto, 1i64)
        }
        2 => {
            // range(from, upto): by=1
            let from = match &args[0] {
                Value::Int(i) => *i,
                Value::Float(f) => f.round() as i64,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "range: from must be a number, got {}",
                        v.type_name()
                    )))
                }
            };
            let upto = match &args[1] {
                Value::Int(i) => *i,
                Value::Float(f) => f.round() as i64,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "range: upto must be a number, got {}",
                        v.type_name()
                    )))
                }
            };
            (from, upto, 1i64)
        }
        3 => {
            // range(from, upto, by)
            let from = match &args[0] {
                Value::Int(i) => *i,
                Value::Float(f) => f.round() as i64,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "range: from must be a number, got {}",
                        v.type_name()
                    )))
                }
            };
            let upto = match &args[1] {
                Value::Int(i) => *i,
                Value::Float(f) => f.round() as i64,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "range: upto must be a number, got {}",
                        v.type_name()
                    )))
                }
            };
            let by = match &args[2] {
                Value::Int(i) => *i,
                Value::Float(f) => f.round() as i64,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "range: by must be a number, got {}",
                        v.type_name()
                    )))
                }
            };
            if by == 0 {
                return Err(EvalError::TypeError(
                    "range: step cannot be zero".to_string(),
                ));
            }
            (from, upto, by)
        }
        _ => {
            return Err(EvalError::TypeError(
                "range: expected 1, 2, or 3 arguments".to_string(),
            ))
        }
    };

    let mut result = Vec::new();

    if by > 0 {
        // Positive step: from < upto
        let mut current = from;
        while current < upto {
            result.push(Value::Int(current));
            current += by;
        }
    } else {
        // Negative step: from > upto
        let mut current = from;
        while current > upto {
            result.push(Value::Int(current));
            current += by; // by is negative
        }
    }

    Ok(Value::List(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::HashMapExt;

    #[test]
    fn test_replace_nth_basic() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let index = Value::Int(1);
        let new_value = Value::Int(99);

        let result = replace_nth(&[list.clone(), index, new_value]).unwrap();

        assert_eq!(
            result,
            Value::List(vec![Value::Int(1), Value::Int(99), Value::Int(3),])
        );
    }

    #[test]
    fn test_replace_nth_first_element() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = replace_nth(&[list, Value::Int(0), Value::Int(100)]).unwrap();

        assert_eq!(
            result,
            Value::List(vec![Value::Int(100), Value::Int(2), Value::Int(3),])
        );
    }

    #[test]
    fn test_replace_nth_last_element() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = replace_nth(&[list, Value::Int(2), Value::Int(99)]).unwrap();

        assert_eq!(
            result,
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(99),])
        );
    }

    #[test]
    fn test_replace_nth_does_not_modify_original() {
        let original = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let _result =
            replace_nth(&[original.clone(), Value::Int(0), Value::Int(100)]).unwrap();

        // Original list should be unchanged
        assert_eq!(
            original,
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3),])
        );
    }

    #[test]
    fn test_replace_nth_with_float_index() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = replace_nth(&[list, Value::Float(1.0), Value::Int(99)]).unwrap();

        assert_eq!(
            result,
            Value::List(vec![Value::Int(1), Value::Int(99), Value::Int(3),])
        );
    }

    #[test]
    fn test_replace_nth_out_of_bounds() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = replace_nth(&[list, Value::Int(5), Value::Int(99)]);

        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("out of bounds"));
    }

    #[test]
    fn test_replace_nth_with_null_list() {
        let result = replace_nth(&[Value::Null, Value::Int(0), Value::Int(99)]).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_replace_nth_with_non_list() {
        let result = replace_nth(&[Value::Int(42), Value::Int(0), Value::Int(99)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_nth_with_non_numeric_index() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let result =
            replace_nth(&[list, Value::String("hello".to_string()), Value::Int(99)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_nth_nested_list() {
        let list = Value::List(vec![
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::List(vec![Value::Int(3), Value::Int(4)]),
        ]);
        let new_value = Value::List(vec![Value::Int(99), Value::Int(100)]);
        let result = replace_nth(&[list, Value::Int(0), new_value]).unwrap();

        assert_eq!(
            result,
            Value::List(vec![
                Value::List(vec![Value::Int(99), Value::Int(100)]),
                Value::List(vec![Value::Int(3), Value::Int(4)]),
            ])
        );
    }

    // Tests for map, filter, reduce with lambdas
    #[test]
    fn test_map() {
        use crate::libs::expr::parser::ast::{BinaryOp, Expr};
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        // Test map with lambda: map([1, 2, 3], |x| x * 2) -> [2, 4, 6]
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::Binary {
                op: BinaryOp::Mul,
                left: Box::new(Expr::LambdaParam("x".to_string())),
                right: Box::new(Expr::Int(2)),
            },
        });
        let result = map(&[list, lambda.clone()]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::List(vals) => {
                assert_eq!(vals.len(), 3);
                assert_eq!(vals[0], Value::Int(2));
                assert_eq!(vals[1], Value::Int(4));
                assert_eq!(vals[2], Value::Int(6));
            }
            _ => panic!("Expected List"),
        }

        // Empty list
        let empty_list = Value::List(vec![]);
        let lambda2 = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });
        let result = map(&[empty_list, lambda2]);
        assert_eq!(result.unwrap(), Value::List(vec![]));

        // Null returns null
        let result = map(&[Value::Null, lambda.clone()]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_filter() {
        use crate::libs::expr::parser::ast::{BinaryOp, Expr};
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        // Test filter with lambda: filter([1, 2, 3, 4], |x| x > 2) -> [3, 4]
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::Binary {
                op: BinaryOp::Gt,
                left: Box::new(Expr::LambdaParam("x".to_string())),
                right: Box::new(Expr::Int(2)),
            },
        });
        let result = filter(&[list, lambda.clone()]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::List(vals) => {
                assert_eq!(vals.len(), 2);
                assert_eq!(vals[0], Value::Int(3));
                assert_eq!(vals[1], Value::Int(4));
            }
            _ => panic!("Expected List"),
        }

        // Empty list
        let empty_list = Value::List(vec![]);
        let lambda2 = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::Bool(true),
        });
        let result = filter(&[empty_list, lambda2]);
        assert_eq!(result.unwrap(), Value::List(vec![]));

        // Null returns null
        let result = filter(&[Value::Null, lambda.clone()]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_reduce() {
        use crate::libs::expr::parser::ast::{BinaryOp, Expr};
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        // Sum: |acc, x| acc + x
        let sum_lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["acc".to_string(), "x".to_string()],
            body: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::LambdaParam("acc".to_string())),
                right: Box::new(Expr::LambdaParam("x".to_string())),
            },
        });
        let result = reduce(&[
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
            Value::Int(0),
            sum_lambda,
        ]);
        assert_eq!(result.unwrap(), Value::Int(6));

        // Product: |acc, x| acc * x
        let mul_lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["acc".to_string(), "x".to_string()],
            body: Expr::Binary {
                op: BinaryOp::Mul,
                left: Box::new(Expr::LambdaParam("acc".to_string())),
                right: Box::new(Expr::LambdaParam("x".to_string())),
            },
        });
        let result = reduce(&[
            Value::List(vec![Value::Int(2), Value::Int(3), Value::Int(4)]),
            Value::Int(1),
            mul_lambda,
        ]);
        assert_eq!(result.unwrap(), Value::Int(24));

        // Empty list returns initial value
        let sum_lambda2 = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["acc".to_string(), "x".to_string()],
            body: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::LambdaParam("acc".to_string())),
                right: Box::new(Expr::LambdaParam("x".to_string())),
            },
        });
        let result = reduce(&[Value::List(vec![]), Value::Int(42), sum_lambda2]);
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_sort_by() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        // Test sort_by with string length: sort_by(["cherry", "apple", "pear"], s => len(s))
        let len_lambda = Value::Lambda(LambdaValue {
            params: vec!["s".to_string()],
            body: Expr::Call {
                name: "len".to_string(),
                args: vec![Expr::LambdaParam("s".to_string())],
            },
            captured_vars: HashMap::new(),
        });
        let result = sort_by(&[
            Value::List(vec![
                Value::String("cherry".to_string()),
                Value::String("apple".to_string()),
                Value::String("pear".to_string()),
            ]),
            len_lambda,
        ]);
        assert_eq!(
            result.unwrap(),
            Value::List(vec![
                Value::String("pear".to_string()),
                Value::String("apple".to_string()),
                Value::String("cherry".to_string()),
            ])
        );

        // Test sort_by with absolute value
        let abs_lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::Call {
                name: "abs".to_string(),
                args: vec![Expr::LambdaParam("x".to_string())],
            },
            captured_vars: HashMap::new(),
        });
        let result = sort_by(&[
            Value::List(vec![
                Value::Int(-5),
                Value::Int(3),
                Value::Int(-1),
                Value::Int(4),
            ]),
            abs_lambda,
        ]);
        assert_eq!(
            result.unwrap(),
            Value::List(vec![
                Value::Int(-1),
                Value::Int(3),
                Value::Int(4),
                Value::Int(-5),
            ])
        );

        // Test sort_by with null list
        let identity_lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = sort_by(&[Value::Null, identity_lambda]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_join() {
        assert_eq!(
            join(&[
                Value::List(vec![
                    Value::String("a".to_string()),
                    Value::String("b".to_string()),
                    Value::String("c".to_string()),
                ]),
                Value::String(",".to_string()),
            ])
            .unwrap(),
            Value::String("a,b,c".to_string())
        );
    }

    #[test]
    fn test_first() {
        assert_eq!(
            first(&[Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])])
            .unwrap(),
            Value::Int(1)
        );
    }

    #[test]
    fn test_last() {
        assert_eq!(
            last(&[Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])])
            .unwrap(),
            Value::Int(3)
        );
    }

    #[test]
    fn test_reverse() {
        assert_eq!(
            reverse(&[Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])])
            .unwrap(),
            Value::List(vec![Value::Int(3), Value::Int(2), Value::Int(1)])
        );
        // Empty list
        assert_eq!(
            reverse(&[Value::List(vec![])]).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_nth() {
        let list = Value::List(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
        assert_eq!(nth(&[list.clone(), Value::Int(0)]).unwrap(), Value::Int(10));
        assert_eq!(nth(&[list.clone(), Value::Int(2)]).unwrap(), Value::Int(30));
        // Out of bounds
        assert_eq!(nth(&[list.clone(), Value::Int(5)]).unwrap(), Value::Null);
    }

    #[test]
    fn test_sort() {
        assert_eq!(
            sort(&[Value::List(vec![
                Value::Int(3),
                Value::Int(1),
                Value::Int(2),
            ])])
            .unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
        // Mixed types
        assert_eq!(
            sort(&[Value::List(vec![
                Value::Float(2.5),
                Value::Int(1),
                Value::Float(1.5),
            ])])
            .unwrap(),
            Value::List(vec![Value::Int(1), Value::Float(1.5), Value::Float(2.5)])
        );
    }

    #[test]
    fn test_unique() {
        assert_eq!(
            unique(&[Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(1),
                Value::Int(3),
            ])])
            .unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
        // Strings
        assert_eq!(
            unique(&[Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
                Value::String("a".to_string()),
            ])])
            .unwrap(),
            Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
            ])
        );
    }

    #[test]
    fn test_slice() {
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ]);
        // slice(list, 1, 3) -> [2, 3]
        assert_eq!(
            slice(&[list.clone(), Value::Int(1), Value::Int(3)]).unwrap(),
            Value::List(vec![Value::Int(2), Value::Int(3)])
        );
        // slice(list, 2) -> [3, 4, 5]
        assert_eq!(
            slice(&[list.clone(), Value::Int(2)]).unwrap(),
            Value::List(vec![Value::Int(3), Value::Int(4), Value::Int(5)])
        );
        // Out of bounds handling
        assert_eq!(
            slice(&[list.clone(), Value::Int(10)]).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_range() {
        // range(4) -> [0, 1, 2, 3]
        assert_eq!(
            range(&[Value::Int(4)]).unwrap(),
            Value::List(vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
                Value::Int(3)
            ])
        );
        // range(2, 4) -> [2, 3]
        assert_eq!(
            range(&[Value::Int(2), Value::Int(4)]).unwrap(),
            Value::List(vec![Value::Int(2), Value::Int(3)])
        );
        // range(0, 10, 3) -> [0, 3, 6, 9]
        assert_eq!(
            range(&[Value::Int(0), Value::Int(10), Value::Int(3)]).unwrap(),
            Value::List(vec![
                Value::Int(0),
                Value::Int(3),
                Value::Int(6),
                Value::Int(9)
            ])
        );
        // range(0, -5, -1) -> [0, -1, -2, -3, -4]
        assert_eq!(
            range(&[Value::Int(0), Value::Int(-5), Value::Int(-1)]).unwrap(),
            Value::List(vec![
                Value::Int(0),
                Value::Int(-1),
                Value::Int(-2),
                Value::Int(-3),
                Value::Int(-4)
            ])
        );
        // range(0, 10, -1) -> [] (step direction doesn't match range direction)
        assert_eq!(
            range(&[Value::Int(0), Value::Int(10), Value::Int(-1)]).unwrap(),
            Value::List(vec![])
        );
        // range with float arguments (rounded to nearest integer)
        assert_eq!(
            range(&[Value::Float(2.7), Value::Float(5.2)]).unwrap(),
            Value::List(vec![Value::Int(3), Value::Int(4)])
        );
    }

    #[test]
    fn test_range_errors() {
        // Step cannot be zero
        assert!(range(&[Value::Int(0), Value::Int(10), Value::Int(0)]).is_err());
        // Invalid argument type
        assert!(range(&[Value::String("hello".to_string())]).is_err());
    }

    // Additional tests for join
    #[test]
    fn test_join_empty_list() {
        assert_eq!(
            join(&[Value::List(vec![]), Value::String(",".to_string())]).unwrap(),
            Value::String("".to_string())
        );
    }

    #[test]
    fn test_join_single_element() {
        assert_eq!(
            join(&[
                Value::List(vec![Value::Int(1)]),
                Value::String(",".to_string())
            ])
            .unwrap(),
            Value::String("1".to_string())
        );
    }

    #[test]
    fn test_join_with_null() {
        assert_eq!(
            join(&[Value::Null, Value::String(",".to_string())]).unwrap(),
            Value::Null
        );
    }

    #[test]
    fn test_join_non_list() {
        assert!(join(&[Value::Int(42), Value::String(",".to_string())]).is_err());
    }

    #[test]
    fn test_join_mixed_types() {
        assert_eq!(
            join(&[
                Value::List(vec![
                    Value::Int(1),
                    Value::String("a".to_string()),
                    Value::Bool(true)
                ]),
                Value::String("|".to_string()),
            ])
            .unwrap(),
            Value::String("1|a|true".to_string())
        );
    }

    // Additional tests for first
    #[test]
    fn test_first_empty_list() {
        assert_eq!(first(&[Value::List(vec![])]).unwrap(), Value::Null);
    }

    #[test]
    fn test_first_null() {
        assert_eq!(first(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_first_non_list() {
        assert!(first(&[Value::Int(42)]).is_err());
    }

    // Additional tests for last
    #[test]
    fn test_last_empty_list() {
        assert_eq!(last(&[Value::List(vec![])]).unwrap(), Value::Null);
    }

    #[test]
    fn test_last_null() {
        assert_eq!(last(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_last_non_list() {
        assert!(last(&[Value::Int(42)]).is_err());
    }

    // Additional tests for reverse
    #[test]
    fn test_reverse_null() {
        assert_eq!(reverse(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_reverse_non_list() {
        assert!(reverse(&[Value::Int(42)]).is_err());
    }

    #[test]
    fn test_reverse_single_element() {
        assert_eq!(
            reverse(&[Value::List(vec![Value::Int(1)])]).unwrap(),
            Value::List(vec![Value::Int(1)])
        );
    }

    // Additional tests for nth
    #[test]
    fn test_nth_with_float() {
        let list = Value::List(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
        // 1.7 rounds to 2, so we get the element at index 2
        assert_eq!(nth(&[list, Value::Float(1.7)]).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_nth_null_list() {
        assert_eq!(nth(&[Value::Null, Value::Int(0)]).unwrap(), Value::Null);
    }

    #[test]
    fn test_nth_non_list() {
        assert!(nth(&[Value::Int(42), Value::Int(0)]).is_err());
    }

    #[test]
    fn test_nth_non_numeric_index() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert!(nth(&[list, Value::String("hello".to_string())]).is_err());
    }

    // Additional tests for sort
    #[test]
    fn test_sort_empty_list() {
        assert_eq!(sort(&[Value::List(vec![])]).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_sort_null() {
        assert_eq!(sort(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_sort_non_list() {
        assert!(sort(&[Value::Int(42)]).is_err());
    }

    #[test]
    fn test_sort_strings() {
        assert_eq!(
            sort(&[Value::List(vec![
                Value::String("banana".to_string()),
                Value::String("apple".to_string()),
                Value::String("cherry".to_string()),
            ])])
            .unwrap(),
            Value::List(vec![
                Value::String("apple".to_string()),
                Value::String("banana".to_string()),
                Value::String("cherry".to_string()),
            ])
        );
    }

    #[test]
    fn test_sort_mixed_types() {
        // Mixed types should fall back to string comparison
        let result = sort(&[Value::List(vec![
            Value::String("10".to_string()),
            Value::Int(2),
            Value::String("1".to_string()),
        ])]);
        assert!(result.is_ok());
    }

    // Additional tests for unique
    #[test]
    fn test_unique_empty_list() {
        assert_eq!(unique(&[Value::List(vec![])]).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_unique_null() {
        assert_eq!(unique(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_unique_non_list() {
        assert!(unique(&[Value::Int(42)]).is_err());
    }

    #[test]
    fn test_unique_all_same() {
        assert_eq!(
            unique(&[Value::List(vec![
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
            ])])
            .unwrap(),
            Value::List(vec![Value::Int(1)])
        );
    }

    #[test]
    fn test_unique_all_different() {
        assert_eq!(
            unique(&[Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])])
            .unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }

    // Additional tests for slice
    #[test]
    fn test_slice_empty_list() {
        assert_eq!(
            slice(&[Value::List(vec![]), Value::Int(0)]).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_slice_null() {
        assert_eq!(slice(&[Value::Null, Value::Int(0)]).unwrap(), Value::Null);
    }

    #[test]
    fn test_slice_non_list() {
        assert!(slice(&[Value::Int(42), Value::Int(0)]).is_err());
    }

    #[test]
    fn test_slice_end_less_than_start() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        // When end < start, end should be set to start
        assert_eq!(
            slice(&[list, Value::Int(2), Value::Int(1)]).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_slice_with_float() {
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        // 1.2 rounds to 1, 3.7 rounds to 4
        assert_eq!(
            slice(&[list, Value::Float(1.2), Value::Float(3.7)]).unwrap(),
            Value::List(vec![Value::Int(2), Value::Int(3), Value::Int(4)])
        );
    }

    #[test]
    fn test_slice_non_numeric_start() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert!(slice(&[list, Value::String("hello".to_string())]).is_err());
    }

    // Additional tests for range
    #[test]
    fn test_range_empty() {
        // range(0) -> []
        assert_eq!(range(&[Value::Int(0)]).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_range_negative_only() {
        // range(-3) -> []
        assert_eq!(range(&[Value::Int(-3)]).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_range_same_from_and_upto() {
        // range(5, 5) -> []
        assert_eq!(
            range(&[Value::Int(5), Value::Int(5)]).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_range_large_step() {
        // range(0, 10, 5) -> [0, 5]
        assert_eq!(
            range(&[Value::Int(0), Value::Int(10), Value::Int(5)]).unwrap(),
            Value::List(vec![Value::Int(0), Value::Int(5)])
        );
    }

    #[test]
    fn test_range_negative_step_wrong_direction() {
        // range(10, 0, -1) -> [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]
        let result = range(&[Value::Int(10), Value::Int(0), Value::Int(-1)]).unwrap();
        if let Value::List(vals) = result {
            assert_eq!(vals.len(), 10);
            assert_eq!(vals[0], Value::Int(10));
            assert_eq!(vals[9], Value::Int(1));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_range_invalid_types() {
        // Two args with invalid types
        assert!(range(&[Value::Int(0), Value::String("hello".to_string())]).is_err());
        assert!(range(&[Value::String("hello".to_string()), Value::Int(10)]).is_err());
    }

    #[test]
    fn test_range_three_args_invalid() {
        // Three args with invalid types
        assert!(range(&[
            Value::Int(0),
            Value::Int(10),
            Value::String("hello".to_string())
        ])
        .is_err());
    }

    // Additional tests for map, filter, reduce error cases
    #[test]
    fn test_map_non_lambda() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert!(map(&[list, Value::Int(42)]).is_err());
    }

    #[test]
    fn test_filter_non_lambda() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert!(filter(&[list, Value::Int(42)]).is_err());
    }

    #[test]
    fn test_reduce_non_lambda() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert!(reduce(&[list, Value::Int(0), Value::Int(42)]).is_err());
    }

    #[test]
    fn test_reduce_lambda_insufficient_params() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        // Lambda with only 1 parameter should error
        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });
        let result = reduce(&[Value::List(vec![Value::Int(1)]), Value::Int(0), lambda]);
        assert!(result.is_err());
    }

    #[test]
    fn test_sort_by_non_lambda() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert!(sort_by(&[list, Value::Int(42)]).is_err());
    }

    #[test]
    fn test_sort_by_null() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let identity_lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        assert_eq!(
            sort_by(&[Value::Null, identity_lambda]).unwrap(),
            Value::Null
        );
    }

    #[test]
    fn test_sort_by_non_list() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let identity_lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        assert!(sort_by(&[Value::Int(42), identity_lambda]).is_err());
    }

    // Additional tests for error handling branches

    #[test]
    fn test_map_with_empty_params_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        // Lambda with no parameters should trigger error in apply_lambda
        let empty_params_lambda = Value::Lambda(LambdaValue {
            params: vec![], // Empty params
            body: Expr::Int(42),
            captured_vars: HashMap::new(),
        });
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let result = map(&[list, empty_params_lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("lambda has no parameters"));
    }

    #[test]
    fn test_filter_with_empty_params_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        // Lambda with no parameters should trigger error in apply_lambda
        let empty_params_lambda = Value::Lambda(LambdaValue {
            params: vec![], // Empty params
            body: Expr::Bool(true),
            captured_vars: HashMap::new(),
        });
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let result = filter(&[list, empty_params_lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("lambda has no parameters"));
    }

    #[test]
    fn test_reduce_with_empty_params_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        // Lambda with no parameters - reduce checks for at least 2 params
        let empty_params_lambda = Value::Lambda(LambdaValue {
            params: vec![], // Empty params
            body: Expr::Int(42),
            captured_vars: HashMap::new(),
        });
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let result = reduce(&[list, Value::Int(0), empty_params_lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("lambda must have at least 2 parameters"));
    }

    #[test]
    fn test_sort_by_with_empty_params_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        // Lambda with no parameters should trigger error in apply_lambda
        let empty_params_lambda = Value::Lambda(LambdaValue {
            params: vec![], // Empty params
            body: Expr::Int(1),
            captured_vars: HashMap::new(),
        });
        let list = Value::List(vec![Value::Int(3), Value::Int(1), Value::Int(2)]);
        let result = sort_by(&[list, empty_params_lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("lambda has no parameters"));
    }

    #[test]
    fn test_join_with_datetime() {
        // join with non-list, non-null should fail
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = join(&[dt, Value::String(",".to_string())]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_join_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = join(&[lambda, Value::String(",".to_string())]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_first_with_datetime() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = first(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a list"));
    }

    #[test]
    fn test_first_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = first(&[lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a list"));
    }

    #[test]
    fn test_last_with_datetime() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = last(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a list"));
    }

    #[test]
    fn test_last_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = last(&[lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a list"));
    }

    #[test]
    fn test_reverse_with_datetime() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = reverse(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a list"));
    }

    #[test]
    fn test_reverse_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = reverse(&[lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a list"));
    }

    #[test]
    fn test_sort_with_datetime() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = sort(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a list"));
    }

    #[test]
    fn test_sort_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = sort(&[lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a list"));
    }

    #[test]
    fn test_unique_with_datetime() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = unique(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a list"));
    }

    #[test]
    fn test_unique_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = unique(&[lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a list"));
    }

    #[test]
    fn test_nth_with_datetime() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = nth(&[dt, Value::Int(0)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_nth_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = nth(&[lambda, Value::Int(0)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_slice_with_datetime() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = slice(&[dt, Value::Int(0), Value::Int(1)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_slice_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = slice(&[lambda, Value::Int(0), Value::Int(1)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_replace_nth_with_datetime() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = replace_nth(&[dt, Value::Int(0), Value::Int(99)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_replace_nth_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = replace_nth(&[lambda, Value::Int(0), Value::Int(99)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_map_with_datetime() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;
        use chrono::Utc;

        let dt = Value::DateTime(Utc::now());
        let identity_lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = map(&[dt, identity_lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_filter_with_datetime() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;
        use chrono::Utc;

        let dt = Value::DateTime(Utc::now());
        let identity_lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = filter(&[dt, identity_lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_reduce_with_datetime() {
        use crate::libs::expr::parser::ast::{BinaryOp, Expr};
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;
        use chrono::Utc;

        let dt = Value::DateTime(Utc::now());
        let sum_lambda = Value::Lambda(LambdaValue {
            params: vec!["acc".to_string(), "x".to_string()],
            body: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::LambdaParam("acc".to_string())),
                right: Box::new(Expr::LambdaParam("x".to_string())),
            },
            captured_vars: HashMap::new(),
        });
        let result = reduce(&[dt, Value::Int(0), sum_lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_sort_by_with_datetime() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;
        use chrono::Utc;

        let dt = Value::DateTime(Utc::now());
        let identity_lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = sort_by(&[dt, identity_lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("first argument must be a list"));
    }

    #[test]
    fn test_range_with_datetime() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = range(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a number"));
    }

    #[test]
    fn test_range_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = range(&[lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("argument must be a number"));
    }
}
