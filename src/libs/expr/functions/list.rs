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
fn apply_lambda(
    lambda: &LambdaValue,
    item: &Value,
) -> Result<Value, EvalError> {
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
                a.compare(b)
                    .unwrap_or(std::cmp::Ordering::Equal)
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
