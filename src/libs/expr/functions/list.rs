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

/// Take elements from a list while a lambda condition is true
/// Stops at the first element where the condition is false
pub fn take_while(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let lambda = match &args[1] {
                Value::Lambda(l) => l,
                _ => {
                    return Err(EvalError::TypeError(
                        "take_while: second argument must be a lambda".to_string(),
                    ))
                }
            };

            let mut result = Vec::new();
            for item in list {
                let keep = apply_lambda(lambda, item)?;
                if keep.as_bool() {
                    result.push(item.clone());
                } else {
                    break;
                }
            }

            Ok(Value::List(result))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "take_while: first argument must be a list".to_string(),
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

/// Check if a list is empty
/// is_empty(list) -> bool
pub fn is_empty(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => Ok(Value::Bool(list.is_empty())),
        Value::Null => Ok(Value::Bool(true)),
        _ => Err(EvalError::TypeError(
            "is_empty: argument must be a list".to_string(),
        )),
    }
}

/// Take the first n elements from a list
/// take(list, n) -> list
pub fn take(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let n = match &args[1] {
                Value::Int(i) => (*i).max(0) as usize,
                Value::Float(f) => f.round().max(0.0) as usize,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "take: second argument must be a number, got {}",
                        v.type_name()
                    )))
                }
            };
            let end = n.min(list.len());
            Ok(Value::List(list[..end].to_vec()))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "take: first argument must be a list".to_string(),
        )),
    }
}

/// Drop the first n elements from a list
/// drop(list, n) -> list
pub fn drop(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let n = match &args[1] {
                Value::Int(i) => (*i).max(0) as usize,
                Value::Float(f) => f.round().max(0.0) as usize,
                v => {
                    return Err(EvalError::TypeError(format!(
                        "drop: second argument must be a number, got {}",
                        v.type_name()
                    )))
                }
            };
            let start = n.min(list.len());
            Ok(Value::List(list[start..].to_vec()))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "drop: first argument must be a list".to_string(),
        )),
    }
}

/// Check if a list contains an element
/// contains(list, element) -> bool
pub fn contains(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let target = &args[1];
            // Use string representation for comparison
            let target_str = target.to_string();
            for item in list {
                if item.to_string() == target_str {
                    return Ok(Value::Bool(true));
                }
            }
            Ok(Value::Bool(false))
        }
        Value::Null => Ok(Value::Bool(false)),
        _ => Err(EvalError::TypeError(
            "contains: first argument must be a list".to_string(),
        )),
    }
}

/// Get the length of a list
/// len(list) -> int
pub fn len(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => Ok(Value::Int(list.len() as i64)),
        Value::Null => Ok(Value::Int(0)),
        _ => Err(EvalError::TypeError(
            "len: argument must be a list".to_string(),
        )),
    }
}

/// Flatten a nested list by one level
/// flatten(list) -> list
pub fn flatten(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::List(list) => {
            let mut result = Vec::new();
            for item in list {
                match item {
                    Value::List(sublist) => result.extend(sublist.clone()),
                    _ => result.push(item.clone()),
                }
            }
            Ok(Value::List(result))
        }
        Value::Null => Ok(Value::Null),
        _ => Err(EvalError::TypeError(
            "flatten: argument must be a list".to_string(),
        )),
    }
}

/// Zip multiple lists into a list of tuples (as lists)
/// zip(list1, list2, ...) -> list
pub fn zip(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Ok(Value::List(vec![]));
    }

    // Collect lists, treating null as empty list
    let mut lists: Vec<Vec<Value>> = Vec::new();
    for arg in args {
        match arg {
            Value::List(list) => lists.push(list.clone()),
            Value::Null => lists.push(Vec::new()),
            _ => {
                return Err(EvalError::TypeError(
                    "zip: all arguments must be lists".to_string(),
                ))
            }
        }
    }

    // Find minimum length
    let min_len = lists.iter().map(|l| l.len()).min().unwrap_or(0);

    let mut result = Vec::new();
    for i in 0..min_len {
        let tuple: Vec<Value> = lists.iter().map(|l| l[i].clone()).collect();
        result.push(Value::List(tuple));
    }

    Ok(Value::List(result))
}

/// Partition a list into two lists based on a predicate
/// partition(list, pred) -> [satisfying, not_satisfying]
pub fn partition(args: &[Value]) -> Result<Value, EvalError> {
    let list = match &args[0] {
        Value::List(list) => list,
        Value::Null => return Ok(Value::Null),
        _ => {
            return Err(EvalError::TypeError(
                "partition: first argument must be a list".to_string(),
            ))
        }
    };

    let pred = match &args[1] {
        Value::Lambda(lambda) => lambda,
        _ => {
            return Err(EvalError::TypeError(
                "partition: second argument must be a lambda".to_string(),
            ))
        }
    };

    let mut satisfying = Vec::new();
    let mut not_satisfying = Vec::new();

    for item in list {
        let result = apply_lambda(pred, item)?;
        match result {
            Value::Bool(true) => satisfying.push(item.clone()),
            _ => not_satisfying.push(item.clone()),
        }
    }

    Ok(Value::List(vec![
        Value::List(satisfying),
        Value::List(not_satisfying),
    ]))
}

/// Map a function over a list and flatten the result by one level
/// flat_map(list, f) -> list
pub fn flat_map(args: &[Value]) -> Result<Value, EvalError> {
    let list = match &args[0] {
        Value::List(list) => list,
        Value::Null => return Ok(Value::Null),
        _ => {
            return Err(EvalError::TypeError(
                "flat_map: first argument must be a list".to_string(),
            ))
        }
    };

    let f = match &args[1] {
        Value::Lambda(lambda) => lambda,
        _ => {
            return Err(EvalError::TypeError(
                "flat_map: second argument must be a lambda".to_string(),
            ))
        }
    };

    let mut result = Vec::new();
    for item in list {
        let mapped = apply_lambda(f, item)?;
        match mapped {
            Value::List(sublist) => result.extend(sublist),
            _ => result.push(mapped),
        }
    }

    Ok(Value::List(result))
}

/// Group a list into chunks of size n
/// grouped(list, n) -> list
pub fn grouped(args: &[Value]) -> Result<Value, EvalError> {
    let list = match &args[0] {
        Value::List(list) => list,
        Value::Null => return Ok(Value::Null),
        _ => {
            return Err(EvalError::TypeError(
                "grouped: first argument must be a list".to_string(),
            ))
        }
    };

    let n = match &args[1] {
        Value::Int(i) => (*i).max(1) as usize,
        Value::Float(f) => f.round().max(1.0) as usize,
        _ => {
            return Err(EvalError::TypeError(
                "grouped: second argument must be a number".to_string(),
            ))
        }
    };

    let mut result = Vec::new();
    let mut chunk = Vec::new();

    for (i, item) in list.iter().enumerate() {
        chunk.push(item.clone());
        if chunk.len() == n || i == list.len() - 1 {
            result.push(Value::List(chunk));
            chunk = Vec::new();
        }
    }

    // Handle remaining elements if any
    if !chunk.is_empty() {
        result.push(Value::List(chunk));
    }

    Ok(Value::List(result))
}

/// Concatenate multiple lists
/// concat(list1, list2, ...) -> list
pub fn concat(args: &[Value]) -> Result<Value, EvalError> {
    let mut result = Vec::new();
    for arg in args {
        match arg {
            Value::List(list) => result.extend(list.clone()),
            Value::Null => continue,
            _ => {
                return Err(EvalError::TypeError(
                    "concat: all arguments must be lists".to_string(),
                ))
            }
        }
    }
    Ok(Value::List(result))
}

/// Return indices of elements that satisfy the predicate
/// filter_index(list, lambda) -> list
pub fn filter_index(args: &[Value]) -> Result<Value, EvalError> {
    let list = match &args[0] {
        Value::List(list) => list,
        Value::Null => return Ok(Value::Null),
        _ => {
            return Err(EvalError::TypeError(
                "filter_index: first argument must be a list".to_string(),
            ))
        }
    };

    let pred = match &args[1] {
        Value::Lambda(lambda) => lambda.clone(),
        _ => {
            return Err(EvalError::TypeError(
                "filter_index: second argument must be a lambda".to_string(),
            ))
        }
    };

    let mut result = Vec::new();
    for (i, item) in list.iter().enumerate() {
        let condition = apply_lambda(&pred, item)?;
        if condition.as_bool() {
            result.push(Value::Int(i as i64));
        }
    }

    Ok(Value::List(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::{HashMap, HashMapExt};
    use test_case::test_case;

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
    /// Test join function with various inputs
    #[test_case(
        vec![Value::List(vec![]), Value::String(",".to_string())],
        Ok(Value::String("".to_string())) ;
        "join_empty"
    )]
    #[test_case(
        vec![Value::List(vec![Value::Int(1)]), Value::String(",".to_string())],
        Ok(Value::String("1".to_string())) ;
        "join_single"
    )]
    #[test_case(
        vec![Value::Null, Value::String(",".to_string())],
        Ok(Value::Null) ;
        "join_null"
    )]
    #[test_case(
        vec![Value::Int(42), Value::String(",".to_string())],
        Err(()) ;
        "join_non_list"
    )]
    fn test_join_various(args: Vec<Value>, expected: Result<Value, ()>) {
        let result = join(&args);
        match expected {
            Ok(v) => assert_eq!(result.unwrap(), v),
            Err(_) => assert!(result.is_err()),
        }
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

    /// Test list functions with empty list
    #[test_case(first, vec![Value::List(vec![])], Ok(Value::Null) ; "first_empty")]
    #[test_case(last, vec![Value::List(vec![])], Ok(Value::Null) ; "last_empty")]
    #[test_case(sort, vec![Value::List(vec![])], Ok(Value::List(vec![])) ; "sort_empty")]
    #[test_case(unique, vec![Value::List(vec![])], Ok(Value::List(vec![])) ; "unique_empty")]
    #[test_case(slice, vec![Value::List(vec![]), Value::Int(0)], Ok(Value::List(vec![])) ; "slice_empty")]
    fn test_empty_list<F>(f: F, args: Vec<Value>, expected: Result<Value, ()>)
    where
        F: Fn(&[Value]) -> Result<Value, EvalError>,
    {
        match expected {
            Ok(v) => assert_eq!(f(&args).unwrap(), v),
            Err(_) => assert!(f(&args).is_err()),
        }
    }

    /// Test list functions with null
    #[test_case(first, vec![Value::Null], Ok(Value::Null) ; "first_null")]
    #[test_case(last, vec![Value::Null], Ok(Value::Null) ; "last_null")]
    #[test_case(reverse, vec![Value::Null], Ok(Value::Null) ; "reverse_null")]
    #[test_case(sort, vec![Value::Null], Ok(Value::Null) ; "sort_null")]
    #[test_case(unique, vec![Value::Null], Ok(Value::Null) ; "unique_null")]
    #[test_case(slice, vec![Value::Null, Value::Int(0)], Ok(Value::Null) ; "slice_null")]
    fn test_null_handling<F>(f: F, args: Vec<Value>, expected: Result<Value, ()>)
    where
        F: Fn(&[Value]) -> Result<Value, EvalError>,
    {
        match expected {
            Ok(v) => assert_eq!(f(&args).unwrap(), v),
            Err(_) => assert!(f(&args).is_err()),
        }
    }

    /// Test list functions with non-list input (should error)
    #[test_case(first, vec![Value::Int(42)] ; "first_non_list")]
    #[test_case(last, vec![Value::Int(42)] ; "last_non_list")]
    #[test_case(reverse, vec![Value::Int(42)] ; "reverse_non_list")]
    #[test_case(sort, vec![Value::Int(42)] ; "sort_non_list")]
    #[test_case(unique, vec![Value::Int(42)] ; "unique_non_list")]
    #[test_case(slice, vec![Value::Int(42), Value::Int(0)] ; "slice_non_list")]
    fn test_non_list_error<F>(f: F, args: Vec<Value>)
    where
        F: Fn(&[Value]) -> Result<Value, EvalError>,
    {
        assert!(f(&args).is_err());
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
        assert_eq!(nth(&[list, Value::Float(1.7)]).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_nth_non_numeric_index() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert!(nth(&[list, Value::String("hello".to_string())]).is_err());
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

    /// Test range function with various inputs
    #[test_case(vec![Value::Int(0)], Ok(vec![]) ; "range_zero")]
    #[test_case(vec![Value::Int(-3)], Ok(vec![]) ; "range_negative")]
    #[test_case(vec![Value::Int(5), Value::Int(5)], Ok(vec![]) ; "range_same")]
    #[test_case(vec![Value::Int(0), Value::Int(10), Value::Int(5)], Ok(vec![0, 5]) ; "range_large_step")]
    #[test_case(vec![Value::Int(0), Value::String("hello".to_string())], Err(()) ; "range_invalid_second")]
    #[test_case(vec![Value::String("hello".to_string()), Value::Int(10)], Err(()) ; "range_invalid_first")]
    #[test_case(
        vec![Value::Int(0), Value::Int(10), Value::String("hello".to_string())],
        Err(()) ;
        "range_invalid_third"
    )]
    fn test_range_various(args: Vec<Value>, expected: Result<Vec<i64>, ()>) {
        let result = range(&args);
        match expected {
            Ok(vals) => {
                let expected_list =
                    Value::List(vals.into_iter().map(Value::Int).collect());
                assert_eq!(result.unwrap(), expected_list);
            }
            Err(_) => assert!(result.is_err()),
        }
    }

    #[test]
    fn test_range_negative_step_wrong_direction() {
        let result = range(&[Value::Int(10), Value::Int(0), Value::Int(-1)]).unwrap();
        if let Value::List(vals) = result {
            assert_eq!(vals.len(), 10);
            assert_eq!(vals[0], Value::Int(10));
            assert_eq!(vals[9], Value::Int(1));
        } else {
            panic!("Expected List");
        }
    }

    // Additional tests for map, filter, reduce error cases
    /// Test map/filter/reduce with non-lambda second argument
    #[test_case(map, vec![Value::Int(42)] ; "map_non_lambda")]
    #[test_case(filter, vec![Value::Int(42)] ; "filter_non_lambda")]
    fn test_hof_non_lambda<F>(f: F, second_arg: Vec<Value>)
    where
        F: Fn(&[Value]) -> Result<Value, EvalError>,
    {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let mut args = vec![list];
        args.extend(second_arg);
        assert!(f(&args).is_err());
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

        // Lambda with only 1 parameter should error
        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });
        let result = reduce(&[Value::List(vec![Value::Int(1)]), Value::Int(0), lambda]);
        assert!(result.is_err());
    }

    /// Test sort_by with various error conditions
    #[test]
    fn test_sort_by_error_conditions() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;

        let identity_lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });

        // Test non-lambda
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert!(sort_by(&[list, Value::Int(42)]).is_err());

        // Test null
        assert_eq!(
            sort_by(&[Value::Null, identity_lambda.clone()]).unwrap(),
            Value::Null
        );

        // Test non-list
        assert!(sort_by(&[Value::Int(42), identity_lambda]).is_err());
    }

    // Additional tests for error handling branches
    /// Test map/filter/sort_by with empty params lambda
    #[test_case(
        map,
        vec![Value::List(vec![Value::Int(1), Value::Int(2)])],
        crate::libs::expr::parser::ast::Expr::Int(42),
        "lambda has no parameters" ;
        "map_empty_lambda"
    )]
    #[test_case(
        filter,
        vec![Value::List(vec![Value::Int(1), Value::Int(2)])],
        crate::libs::expr::parser::ast::Expr::Bool(true),
        "lambda has no parameters" ;
        "filter_empty_lambda"
    )]
    #[test_case(
        sort_by,
        vec![Value::List(vec![Value::Int(3), Value::Int(1), Value::Int(2)])],
        crate::libs::expr::parser::ast::Expr::Int(1),
        "lambda has no parameters" ;
        "sort_by_empty_lambda"
    )]
    fn test_empty_params_lambda<F>(
        f: F,
        extra_args: Vec<Value>,
        body: crate::libs::expr::parser::ast::Expr,
        expected_error: &str,
    ) where
        F: Fn(&[Value]) -> Result<Value, EvalError>,
    {
        use crate::libs::expr::runtime::value::LambdaValue;

        let empty_params_lambda = Value::Lambda(LambdaValue {
            params: vec![],
            body,
            captured_vars: HashMap::new(),
        });
        let mut args = extra_args;
        args.push(empty_params_lambda);
        let result = f(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }

    #[test]
    fn test_reduce_with_empty_params_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;

        let empty_params_lambda = Value::Lambda(LambdaValue {
            params: vec![],
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

    /// Test list functions with non-list first argument (datetime)
    #[test]
    fn test_list_fns_with_datetime_simple() {
        use chrono::Utc;

        let dt = Value::DateTime(Utc::now());

        // Test join
        let result = join(&[dt.clone(), Value::String(",".to_string())]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test first
        let result = first(&[dt.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test last
        let result = last(&[dt.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test reverse
        let result = reverse(&[dt]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));
    }

    /// Test list functions with lambda as first argument
    #[test]
    fn test_list_fns_with_lambda_simple() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });

        // Test join
        let result = join(&[lambda.clone(), Value::String(",".to_string())]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test first
        let result = first(&[lambda.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test last
        let result = last(&[lambda.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test reverse
        let result = reverse(&[lambda]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));
    }

    /// Test various list functions with non-list first argument (datetime)
    #[test]
    fn test_list_fns_with_datetime() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use chrono::Utc;

        let dt = Value::DateTime(Utc::now());
        let identity_lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let sum_lambda = Value::Lambda(LambdaValue {
            params: vec!["acc".to_string(), "x".to_string()],
            body: Expr::Binary {
                op: crate::libs::expr::parser::ast::BinaryOp::Add,
                left: Box::new(Expr::LambdaParam("acc".to_string())),
                right: Box::new(Expr::LambdaParam("x".to_string())),
            },
            captured_vars: HashMap::new(),
        });

        // Test sort
        let result = sort(&[dt.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test unique
        let result = unique(&[dt.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test nth
        let result = nth(&[dt.clone(), Value::Int(0)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test slice
        let result = slice(&[dt.clone(), Value::Int(0), Value::Int(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test replace_nth
        let result = replace_nth(&[dt.clone(), Value::Int(0), Value::Int(99)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test map
        let result = map(&[dt.clone(), identity_lambda.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test filter
        let result = filter(&[dt.clone(), identity_lambda.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test reduce
        let result = reduce(&[dt, Value::Int(0), sum_lambda]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));
    }

    /// Test various list functions with lambda as first argument
    #[test]
    fn test_list_fns_with_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });

        // Test sort
        let result = sort(&[lambda.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test unique
        let result = unique(&[lambda.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test nth
        let result = nth(&[lambda.clone(), Value::Int(0)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test slice
        let result = slice(&[lambda.clone(), Value::Int(0), Value::Int(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Test replace_nth
        let result = replace_nth(&[lambda, Value::Int(0), Value::Int(99)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));
    }

    #[test]
    fn test_sort_by_with_datetime() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;

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

    /// Test range with non-numeric arguments
    #[test]
    fn test_range_with_non_numeric() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use chrono::Utc;

        // Test with datetime
        let dt = Value::DateTime(Utc::now());
        let result = range(&[dt]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("number"));

        // Test with lambda
        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let result = range(&[lambda]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("number"));
    }

    /// Test take_while with various scenarios
    #[test_case(
        vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4), Value::Int(5)],
        crate::libs::expr::parser::ast::BinaryOp::Lt,
        4,
        vec![Value::Int(1), Value::Int(2), Value::Int(3)] ;
        "take_while_basic"
    )]
    #[test_case(
        vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        crate::libs::expr::parser::ast::BinaryOp::Lt,
        0,
        vec![] ;
        "take_while_no_match"
    )]
    fn test_take_while_various(
        list_vals: Vec<Value>,
        op: crate::libs::expr::parser::ast::BinaryOp,
        threshold: i64,
        expected: Vec<Value>,
    ) {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::Binary {
                op,
                left: Box::new(Expr::LambdaParam("x".to_string())),
                right: Box::new(Expr::Int(threshold)),
            },
            captured_vars: HashMap::new(),
        });

        let list = Value::List(list_vals);
        let result = take_while(&[list, lambda]).unwrap();
        assert_eq!(result, Value::List(expected));
    }

    #[test]
    fn test_take_while_edge_cases() {
        use crate::libs::expr::parser::ast::{BinaryOp, Expr};
        use crate::libs::expr::runtime::value::LambdaValue;

        let true_lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::Bool(true),
            captured_vars: HashMap::new(),
        });

        // Empty list
        let result = take_while(&[Value::List(vec![]), true_lambda.clone()]).unwrap();
        assert_eq!(result, Value::List(vec![]));

        // All match
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = take_while(&[list.clone(), true_lambda.clone()]).unwrap();
        assert_eq!(result, list);

        // Null
        let result = take_while(&[Value::Null, true_lambda.clone()]).unwrap();
        assert_eq!(result, Value::Null);

        // Stops at first false
        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::Binary {
                op: BinaryOp::Lt,
                left: Box::new(Expr::LambdaParam("x".to_string())),
                right: Box::new(Expr::Int(3)),
            },
            captured_vars: HashMap::new(),
        });
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(5),
            Value::Int(2),
        ]);
        let result = take_while(&[list, lambda]).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(1), Value::Int(2)]));
    }

    #[test]
    fn test_take_while_errors() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;

        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::Bool(true),
            captured_vars: HashMap::new(),
        });

        // Non-list
        let result = take_while(&[Value::Int(42), lambda.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));

        // Non-lambda
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let result = take_while(&[list, Value::Int(42)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("lambda"));
    }

    // Tests for is_empty
    #[test_case(Value::List(vec![]), Ok(Value::Bool(true)) ; "is_empty_empty")]
    #[test_case(Value::List(vec![Value::Int(1)]), Ok(Value::Bool(false)) ; "is_empty_non_empty")]
    #[test_case(Value::Null, Ok(Value::Bool(true)) ; "is_empty_null")]
    fn test_is_empty_various(input: Value, expected: Result<Value, ()>) {
        match expected {
            Ok(v) => assert_eq!(is_empty(&[input]).unwrap(), v),
            Err(_) => assert!(is_empty(&[input]).is_err()),
        }
    }

    #[test]
    fn test_is_empty_type_error() {
        let result = is_empty(&[Value::Int(123)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));
    }

    // Tests for take
    #[test_case(2, vec![Value::Int(1), Value::Int(2)] ; "take_basic")]
    #[test_case(10, vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)] ; "take_more_than_length")]
    #[test_case(0, vec![] ; "take_zero")]
    fn test_take_list(count: i64, expected: Vec<Value>) {
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        assert_eq!(
            take(&[list, Value::Int(count)]).unwrap(),
            Value::List(expected)
        );
    }

    #[test]
    fn test_take_edge_cases() {
        // Float
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(
            take(&[list, Value::Float(2.5)]).unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );

        // Negative
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(take(&[list, Value::Int(-1)]).unwrap(), Value::List(vec![]));

        // Type error
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let result = take(&[list, Value::String("2".to_string())]);
        assert!(result.is_err());
    }

    // Tests for drop
    #[test_case(2, vec![Value::Int(3), Value::Int(4)] ; "drop_basic")]
    #[test_case(10, vec![] ; "drop_more_than_length")]
    #[test_case(0, vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)] ; "drop_zero")]
    fn test_drop_list(count: i64, expected: Vec<Value>) {
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        assert_eq!(
            drop(&[list, Value::Int(count)]).unwrap(),
            Value::List(expected)
        );
    }

    #[test]
    fn test_drop_edge_cases() {
        // Float (2.5 rounds to 3)
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        assert_eq!(
            drop(&[list, Value::Float(2.5)]).unwrap(),
            Value::List(vec![Value::Int(4)])
        );

        // Negative
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(
            drop(&[list, Value::Int(-1)]).unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );

        // Type error
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let result = drop(&[list, Value::String("2".to_string())]);
        assert!(result.is_err());
    }

    // Tests for contains
    #[test]
    fn test_contains_various() {
        // Int list
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(
            contains(&[list.clone(), Value::Int(2)]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            contains(&[list.clone(), Value::Int(5)]).unwrap(),
            Value::Bool(false)
        );

        // String list
        let list = Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]);
        assert_eq!(
            contains(&[list.clone(), Value::String("a".to_string())]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            contains(&[list.clone(), Value::String("c".to_string())]).unwrap(),
            Value::Bool(false)
        );

        // Null
        assert_eq!(
            contains(&[Value::Null, Value::Int(1)]).unwrap(),
            Value::Bool(false)
        );

        // Type error
        let result = contains(&[Value::Int(123), Value::Int(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("list"));
    }

    // Tests for flatten
    #[test]
    fn test_flatten_various() {
        // Basic nested lists
        let nested = Value::List(vec![
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::List(vec![Value::Int(3), Value::Int(4)]),
        ]);
        assert_eq!(
            flatten(&[nested]).unwrap(),
            Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4)
            ])
        );

        // Mix of lists and non-lists
        let mixed = Value::List(vec![
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::Int(3),
            Value::List(vec![Value::Int(4), Value::Int(5)]),
        ]);
        assert_eq!(
            flatten(&[mixed]).unwrap(),
            Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
                Value::Int(5),
            ])
        );

        // Empty list
        assert_eq!(
            flatten(&[Value::List(vec![])]).unwrap(),
            Value::List(vec![])
        );

        // Null
        assert_eq!(flatten(&[Value::Null]).unwrap(), Value::Null);

        // Type error
        let result = flatten(&[Value::Int(123)]);
        assert!(result.is_err());
    }

    // Tests for zip
    #[test]
    fn test_zip_various() {
        // Basic two lists
        let list1 = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]);
        assert_eq!(
            zip(&[list1, list2]).unwrap(),
            Value::List(vec![
                Value::List(vec![Value::Int(1), Value::String("a".to_string())]),
                Value::List(vec![Value::Int(2), Value::String("b".to_string())]),
            ])
        );

        // Three lists
        let list1 = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::List(vec![Value::Int(10), Value::Int(20)]);
        let list3 = Value::List(vec![Value::Int(100), Value::Int(200)]);
        assert_eq!(
            zip(&[list1, list2, list3]).unwrap(),
            Value::List(vec![
                Value::List(vec![Value::Int(1), Value::Int(10), Value::Int(100)]),
                Value::List(vec![Value::Int(2), Value::Int(20), Value::Int(200)]),
            ])
        );

        // Truncates to shortest
        let list1 = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let list2 = Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]);
        assert_eq!(
            zip(&[list1, list2]).unwrap(),
            Value::List(vec![
                Value::List(vec![Value::Int(1), Value::String("a".to_string())]),
                Value::List(vec![Value::Int(2), Value::String("b".to_string())]),
            ])
        );

        // Empty lists
        assert_eq!(
            zip(&[Value::List(vec![]), Value::List(vec![])]).unwrap(),
            Value::List(vec![])
        );

        // No args
        assert_eq!(zip(&[]).unwrap(), Value::List(vec![]));

        // Type error
        let result = zip(&[Value::Int(123), Value::List(vec![])]);
        assert!(result.is_err());
    }

    // Tests for partition
    #[test]
    fn test_partition_various() {
        use crate::libs::expr::parser::ast::{BinaryOp, Expr};
        use crate::libs::expr::runtime::value::LambdaValue;

        // Basic partition
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        // Lambda: x -> x == 2
        let pred = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::Binary {
                op: BinaryOp::Eq,
                left: Box::new(Expr::LambdaParam("x".to_string())),
                right: Box::new(Expr::Int(2)),
            },
        });
        let result = partition(&[list, pred]).unwrap();
        // Should be [[2], [1, 3, 4]]
        match result {
            Value::List(parts) => {
                assert_eq!(parts.len(), 2);
                match &parts[0] {
                    Value::List(satisfying) => {
                        assert_eq!(satisfying.len(), 1);
                        assert_eq!(satisfying[0], Value::Int(2));
                    }
                    _ => panic!("Expected list"),
                }
                match &parts[1] {
                    Value::List(not_satisfying) => {
                        assert_eq!(not_satisfying.len(), 3);
                    }
                    _ => panic!("Expected list"),
                }
            }
            _ => panic!("Expected list of two lists"),
        }

        // Null
        let pred = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::Bool(true),
        });
        assert_eq!(partition(&[Value::Null, pred]).unwrap(), Value::Null);

        // Type error
        let result = partition(&[Value::Int(123), Value::Int(1)]);
        assert!(result.is_err());
    }

    // Tests for flat_map
    #[test]
    fn test_flat_map_various() {
        use crate::libs::expr::parser::ast::{BinaryOp, Expr};
        use crate::libs::expr::runtime::value::LambdaValue;

        // Basic flat_map
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        // Lambda: x -> [x, x * 2]
        let f = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::List(vec![
                Expr::LambdaParam("x".to_string()),
                Expr::Binary {
                    op: BinaryOp::Mul,
                    left: Box::new(Expr::LambdaParam("x".to_string())),
                    right: Box::new(Expr::Int(2)),
                },
            ]),
        });
        let result = flat_map(&[list, f]).unwrap();
        // Should be [1, 2, 2, 4]
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 4);
                assert_eq!(items[0], Value::Int(1));
                assert_eq!(items[1], Value::Int(2));
                assert_eq!(items[2], Value::Int(2));
                assert_eq!(items[3], Value::Int(4));
            }
            _ => panic!("Expected list"),
        }

        // Null
        let f = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::List(vec![]),
        });
        assert_eq!(flat_map(&[Value::Null, f]).unwrap(), Value::Null);

        // Type error
        let result = flat_map(&[Value::Int(123), Value::Int(1)]);
        assert!(result.is_err());
    }

    // Tests for grouped
    #[test]
    fn test_grouped_basic() {
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        assert_eq!(
            grouped(&[list, Value::Int(2)]).unwrap(),
            Value::List(vec![
                Value::List(vec![Value::Int(1), Value::Int(2)]),
                Value::List(vec![Value::Int(3), Value::Int(4)]),
            ])
        );
    }

    #[test]
    fn test_grouped_with_remainder() {
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ]);
        assert_eq!(
            grouped(&[list, Value::Int(2)]).unwrap(),
            Value::List(vec![
                Value::List(vec![Value::Int(1), Value::Int(2)]),
                Value::List(vec![Value::Int(3), Value::Int(4)]),
                Value::List(vec![Value::Int(5)]),
            ])
        );
    }

    #[test]
    fn test_grouped_with_float() {
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        assert_eq!(
            grouped(&[list, Value::Float(2.0)]).unwrap(),
            Value::List(vec![
                Value::List(vec![Value::Int(1), Value::Int(2)]),
                Value::List(vec![Value::Int(3), Value::Int(4)]),
            ])
        );
    }

    #[test]
    fn test_grouped_zero_becomes_one() {
        // Zero or negative should become 1
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(
            grouped(&[list.clone(), Value::Int(0)]).unwrap(),
            Value::List(vec![
                Value::List(vec![Value::Int(1)]),
                Value::List(vec![Value::Int(2)]),
                Value::List(vec![Value::Int(3)]),
            ])
        );
    }

    #[test]
    fn test_grouped_null() {
        assert_eq!(grouped(&[Value::Null, Value::Int(2)]).unwrap(), Value::Null);
    }

    #[test]
    fn test_grouped_type_error() {
        let result = grouped(&[Value::Int(123), Value::Int(2)]);
        assert!(result.is_err());
    }

    // Tests for concat
    #[test]
    fn test_concat_lists() {
        let list1 = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::List(vec![Value::Int(3), Value::Int(4)]);
        assert_eq!(
            concat(&[list1, list2]).unwrap(),
            Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4)
            ])
        );
    }

    #[test]
    fn test_concat_three_lists() {
        let list1 = Value::List(vec![Value::Int(1)]);
        let list2 = Value::List(vec![Value::Int(2)]);
        let list3 = Value::List(vec![Value::Int(3)]);
        assert_eq!(
            concat(&[list1, list2, list3]).unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }

    #[test]
    fn test_concat_empty_lists() {
        assert_eq!(
            concat(&[Value::List(vec![]), Value::List(vec![])]).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn test_concat_with_null() {
        let list1 = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::List(vec![Value::Int(3), Value::Int(4)]);
        assert_eq!(
            concat(&[list1, Value::Null, list2]).unwrap(),
            Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4)
            ])
        );
    }

    #[test]
    fn test_concat_empty() {
        assert_eq!(concat(&[]).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_concat_type_error() {
        let result = concat(&[Value::Int(123), Value::List(vec![])]);
        assert!(result.is_err());
    }

    // Tests for filter_index
    #[test]
    fn test_filter_index_basic() {
        use crate::libs::expr::parser::ast::{BinaryOp, Expr};
        use crate::libs::expr::runtime::value::LambdaValue;

        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ]);
        // Find indices of even numbers
        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::Binary {
                op: BinaryOp::Eq,
                left: Box::new(Expr::Binary {
                    op: BinaryOp::Mod,
                    left: Box::new(Expr::LambdaParam("x".to_string())),
                    right: Box::new(Expr::Int(2)),
                }),
                right: Box::new(Expr::Int(0)),
            },
        });
        assert_eq!(
            filter_index(&[list, lambda]).unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(3)])
        );
    }

    #[test]
    fn test_filter_index_empty_result() {
        use crate::libs::expr::parser::ast::{BinaryOp, Expr};
        use crate::libs::expr::runtime::value::LambdaValue;

        let list = Value::List(vec![Value::Int(1), Value::Int(3), Value::Int(5)]);
        // Find indices of even numbers (none)
        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::Binary {
                op: BinaryOp::Eq,
                left: Box::new(Expr::Binary {
                    op: BinaryOp::Mod,
                    left: Box::new(Expr::LambdaParam("x".to_string())),
                    right: Box::new(Expr::Int(2)),
                }),
                right: Box::new(Expr::Int(0)),
            },
        });
        assert_eq!(filter_index(&[list, lambda]).unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_filter_index_null() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;

        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::Bool(true),
        });
        assert_eq!(filter_index(&[Value::Null, lambda]).unwrap(), Value::Null);
    }

    #[test]
    fn test_filter_index_type_error() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;

        let result = filter_index(&[
            Value::Int(123),
            Value::Lambda(LambdaValue {
                captured_vars: HashMap::new(),
                params: vec!["x".to_string()],
                body: Expr::Bool(true),
            }),
        ]);
        assert!(result.is_err());
    }

    // Additional tests to improve coverage

    #[test]
    fn test_len_with_null() {
        assert_eq!(len(&[Value::Null]).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_len_type_error() {
        let result = len(&[Value::Int(123)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a list"));
    }

    #[test]
    fn test_take_with_null() {
        assert_eq!(take(&[Value::Null, Value::Int(2)]).unwrap(), Value::Null);
    }

    #[test]
    fn test_take_type_error() {
        let result = take(&[Value::Int(123), Value::Int(2)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a list"));
    }

    #[test]
    fn test_drop_with_null() {
        assert_eq!(drop(&[Value::Null, Value::Int(2)]).unwrap(), Value::Null);
    }

    #[test]
    fn test_drop_type_error() {
        let result = drop(&[Value::Int(123), Value::Int(2)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a list"));
    }

    #[test]
    fn test_slice_non_numeric_end() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = slice(&[list, Value::Int(0), Value::String("hello".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reduce_null_list_returns_initial() {
        let result = reduce(&[Value::Null, Value::Int(42), Value::Int(1)]);
        // Should return initial value when list is null
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_zip_with_null_element() {
        let list1 = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::List(vec![Value::Int(10), Value::Int(20)]);
        // Null should be treated as empty list
        let result = zip(&[list1, Value::Null, list2]);
        // Since null is treated as empty, result should be empty
        assert_eq!(result.unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_partition_non_lambda_error() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let result = partition(&[list, Value::Int(42)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a lambda"));
    }

    #[test]
    fn test_flat_map_non_lambda_error() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let result = flat_map(&[list, Value::Int(42)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a lambda"));
    }

    #[test]
    fn test_flat_map_returns_non_list() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;

        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        // Lambda returns non-list value
        let f = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::Binary {
                op: crate::libs::expr::parser::ast::BinaryOp::Mul,
                left: Box::new(Expr::LambdaParam("x".to_string())),
                right: Box::new(Expr::Int(2)),
            },
        });
        let result = flat_map(&[list, f]).unwrap();
        // Should flatten by pushing non-list values directly
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], Value::Int(2));
                assert_eq!(items[1], Value::Int(4));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_grouped_non_numeric_error() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let result = grouped(&[list, Value::String("hello".to_string())]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a number"));
    }

    #[test]
    fn test_nth_with_float_index() {
        let list = Value::List(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
        // 1.4 rounds to 1
        assert_eq!(nth(&[list, Value::Float(1.4)]).unwrap(), Value::Int(20));
    }

    #[test]
    fn test_range_with_float_single_arg() {
        // range(3.7) should round to 4
        let result = range(&[Value::Float(3.7)]).unwrap();
        match result {
            Value::List(vals) => {
                assert_eq!(vals.len(), 4);
                assert_eq!(vals[0], Value::Int(0));
                assert_eq!(vals[3], Value::Int(3));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_range_with_float_from() {
        // range(1.2, 4) -> [1, 2, 3]
        let result = range(&[Value::Float(1.2), Value::Int(4)]).unwrap();
        match result {
            Value::List(vals) => {
                assert_eq!(vals.len(), 3);
                assert_eq!(vals[0], Value::Int(1));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_range_with_float_upto() {
        // range(0, 3.7) -> [0, 1, 2, 3]
        let result = range(&[Value::Int(0), Value::Float(3.7)]).unwrap();
        match result {
            Value::List(vals) => {
                assert_eq!(vals.len(), 4);
                assert_eq!(vals[3], Value::Int(3));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_range_with_float_step() {
        // range(0, 10, 2.5) -> 2.5 rounds to 3, so [0, 3, 6, 9]
        let result = range(&[Value::Int(0), Value::Int(10), Value::Float(2.5)]).unwrap();
        match result {
            Value::List(vals) => {
                assert_eq!(vals.len(), 4);
                assert_eq!(vals[0], Value::Int(0));
                assert_eq!(vals[1], Value::Int(3));
                assert_eq!(vals[3], Value::Int(9));
            }
            _ => panic!("Expected list"),
        }
    }
}
