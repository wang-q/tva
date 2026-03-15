use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;

pub fn abs(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        Value::String(s) => {
            if let Ok(n) = s.parse::<i64>() {
                Ok(Value::Int(n.abs()))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Float(f.abs()))
            } else {
                Err(EvalError::TypeError(format!(
                    "abs: cannot convert '{}' to number",
                    s
                )))
            }
        }
        Value::Null => Ok(Value::Null),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::List(_) => Err(EvalError::TypeError(
            "abs: cannot convert list to number".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "abs: cannot convert datetime to number".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "abs: cannot convert lambda to number".to_string(),
        )),
    }
}

pub fn round(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(f.round() as i64)),
        Value::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Int(f.round() as i64))
            } else {
                Err(EvalError::TypeError(format!(
                    "round: cannot convert '{}' to number",
                    s
                )))
            }
        }
        Value::Null => Ok(Value::Null),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::List(_) => Err(EvalError::TypeError(
            "round: cannot convert list to number".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "round: cannot convert datetime to number".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "round: cannot convert lambda to number".to_string(),
        )),
    }
}

pub fn min(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::WrongArity {
            name: "min".to_string(),
            expected: 1,
            got: 0,
        });
    }

    let mut min_val: Option<f64> = None;
    for arg in args {
        let val = match arg {
            Value::Int(n) => *n as f64,
            Value::Float(f) => *f,
            Value::String(s) => s.parse::<f64>().map_err(|_| {
                EvalError::TypeError(format!("min: cannot convert '{}' to number", s))
            })?,
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Null => continue,
            Value::List(_) => continue,
            Value::DateTime(_) => continue,
            Value::Lambda(_) => continue,
        };
        min_val = Some(min_val.map_or(val, |m| m.min(val)));
    }

    match min_val {
        Some(v) if v == v.floor() => Ok(Value::Int(v as i64)),
        Some(v) => Ok(Value::Float(v)),
        None => Ok(Value::Null),
    }
}

pub fn max(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::WrongArity {
            name: "max".to_string(),
            expected: 1,
            got: 0,
        });
    }

    let mut max_val: Option<f64> = None;
    for arg in args {
        let val = match arg {
            Value::Int(n) => *n as f64,
            Value::Float(f) => *f,
            Value::String(s) => s.parse::<f64>().map_err(|_| {
                EvalError::TypeError(format!("max: cannot convert '{}' to number", s))
            })?,
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Null => continue,
            Value::List(_) => continue,
            Value::DateTime(_) => continue,
            Value::Lambda(_) => continue,
        };
        max_val = Some(max_val.map_or(val, |m| m.max(val)));
    }

    match max_val {
        Some(v) if v == v.floor() => Ok(Value::Int(v as i64)),
        Some(v) => Ok(Value::Float(v)),
        None => Ok(Value::Null),
    }
}

pub fn int(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::String(s) => {
            if let Ok(n) = s.parse::<i64>() {
                Ok(Value::Int(n))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Int(f as i64))
            } else {
                Err(EvalError::TypeError(format!(
                    "int: cannot convert '{}' to integer",
                    s
                )))
            }
        }
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::Null => Ok(Value::Null),
        Value::List(_) => Err(EvalError::TypeError(
            "int: cannot convert list to integer".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "int: cannot convert datetime to integer".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "int: cannot convert lambda to integer".to_string(),
        )),
    }
}

pub fn float(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
            EvalError::TypeError(format!("float: cannot convert '{}' to float", s))
        }),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Value::Null => Ok(Value::Null),
        Value::List(_) => Err(EvalError::TypeError(
            "float: cannot convert list to float".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "float: cannot convert datetime to float".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "float: cannot convert lambda to float".to_string(),
        )),
    }
}

pub fn ceil(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(f.ceil() as i64)),
        Value::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Int(f.ceil() as i64))
            } else {
                Err(EvalError::TypeError(format!(
                    "ceil: cannot convert '{}' to number",
                    s
                )))
            }
        }
        Value::Null => Ok(Value::Null),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::List(_) => Err(EvalError::TypeError(
            "ceil: cannot convert list to number".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "ceil: cannot convert datetime to number".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "ceil: cannot convert lambda to number".to_string(),
        )),
    }
}

pub fn floor(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(f.floor() as i64)),
        Value::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Int(f.floor() as i64))
            } else {
                Err(EvalError::TypeError(format!(
                    "floor: cannot convert '{}' to number",
                    s
                )))
            }
        }
        Value::Null => Ok(Value::Null),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::List(_) => Err(EvalError::TypeError(
            "floor: cannot convert list to number".to_string(),
        )),
        Value::DateTime(_) => Err(EvalError::TypeError(
            "floor: cannot convert datetime to number".to_string(),
        )),
        Value::Lambda(_) => Err(EvalError::TypeError(
            "floor: cannot convert lambda to number".to_string(),
        )),
    }
}

pub fn sqrt(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("sqrt: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "sqrt: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "sqrt: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "sqrt: cannot convert lambda to number".to_string(),
            ))
        }
    };
    if n < 0.0 {
        return Err(EvalError::TypeError(
            "sqrt: cannot compute square root of negative number".to_string(),
        ));
    }
    Ok(Value::Float(n.sqrt()))
}

pub fn pow(args: &[Value]) -> Result<Value, EvalError> {
    let base = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("pow: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert lambda to number".to_string(),
            ))
        }
    };
    let exp = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("pow: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "pow: cannot convert lambda to number".to_string(),
            ))
        }
    };
    Ok(Value::Float(base.powf(exp)))
}

// Trigonometric functions
pub fn sin(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("sin: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "sin: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "sin: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "sin: cannot convert lambda to number".to_string(),
            ))
        }
    };
    Ok(Value::Float(n.sin()))
}

pub fn cos(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("cos: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "cos: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "cos: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "cos: cannot convert lambda to number".to_string(),
            ))
        }
    };
    Ok(Value::Float(n.cos()))
}

pub fn tan(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("tan: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "tan: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "tan: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "tan: cannot convert lambda to number".to_string(),
            ))
        }
    };
    Ok(Value::Float(n.tan()))
}

// Logarithmic and exponential functions
pub fn ln(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("ln: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "ln: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "ln: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "ln: cannot convert lambda to number".to_string(),
            ))
        }
    };
    if n <= 0.0 {
        return Err(EvalError::TypeError(
            "ln: cannot compute logarithm of non-positive number".to_string(),
        ));
    }
    Ok(Value::Float(n.ln()))
}

pub fn log10(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("log10: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "log10: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "log10: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "log10: cannot convert lambda to number".to_string(),
            ))
        }
    };
    if n <= 0.0 {
        return Err(EvalError::TypeError(
            "log10: cannot compute logarithm of non-positive number".to_string(),
        ));
    }
    Ok(Value::Float(n.log10()))
}

pub fn exp(args: &[Value]) -> Result<Value, EvalError> {
    let n = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            EvalError::TypeError(format!("exp: cannot convert '{}' to number", s))
        })?,
        Value::Null => return Ok(Value::Null),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::List(_) => {
            return Err(EvalError::TypeError(
                "exp: cannot convert list to number".to_string(),
            ))
        }
        Value::DateTime(_) => {
            return Err(EvalError::TypeError(
                "exp: cannot convert datetime to number".to_string(),
            ))
        }
        Value::Lambda(_) => {
            return Err(EvalError::TypeError(
                "exp: cannot convert lambda to number".to_string(),
            ))
        }
    };
    Ok(Value::Float(n.exp()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::HashMapExt;

    #[test]
    fn test_abs() {
        assert_eq!(abs(&[Value::Int(-5)]).unwrap(), Value::Int(5));
        assert_eq!(abs(&[Value::Float(-3.5)]).unwrap(), Value::Float(3.5));
        assert_eq!(abs(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_round() {
        assert_eq!(round(&[Value::Float(3.7)]).unwrap(), Value::Int(4));
        assert_eq!(round(&[Value::Float(3.2)]).unwrap(), Value::Int(3));
        assert_eq!(round(&[Value::Int(5)]).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_min() {
        assert_eq!(
            min(&[Value::Int(3), Value::Int(1), Value::Int(2)]).unwrap(),
            Value::Int(1)
        );
        assert_eq!(
            min(&[Value::Float(3.5), Value::Float(1.2), Value::Float(2.8)]).unwrap(),
            Value::Float(1.2)
        );
        assert_eq!(min(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_max() {
        assert_eq!(
            max(&[Value::Int(3), Value::Int(5), Value::Int(2)]).unwrap(),
            Value::Int(5)
        );
        assert_eq!(
            max(&[Value::Float(3.5), Value::Float(5.2), Value::Float(2.8)]).unwrap(),
            Value::Float(5.2)
        );
        assert_eq!(max(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_int() {
        assert_eq!(int(&[Value::Float(3.7)]).unwrap(), Value::Int(3));
        assert_eq!(
            int(&[Value::String("42".to_string())]).unwrap(),
            Value::Int(42)
        );
        assert_eq!(int(&[Value::Int(42)]).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_float() {
        assert_eq!(float(&[Value::Int(42)]).unwrap(), Value::Float(42.0));
        assert_eq!(
            float(&[Value::String("3.14".to_string())]).unwrap(),
            Value::Float(3.14)
        );
    }

    #[test]
    fn test_ceil() {
        assert_eq!(ceil(&[Value::Float(3.2)]).unwrap(), Value::Int(4));
        assert_eq!(ceil(&[Value::Float(-3.7)]).unwrap(), Value::Int(-3));
    }

    #[test]
    fn test_floor() {
        assert_eq!(floor(&[Value::Float(3.7)]).unwrap(), Value::Int(3));
        assert_eq!(floor(&[Value::Float(-3.2)]).unwrap(), Value::Int(-4));
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(sqrt(&[Value::Float(16.0)]).unwrap(), Value::Float(4.0));
        assert_eq!(sqrt(&[Value::Int(9)]).unwrap(), Value::Float(3.0));
        // Negative number error
        assert!(sqrt(&[Value::Float(-4.0)]).is_err());
    }

    #[test]
    fn test_pow() {
        assert_eq!(
            pow(&[Value::Float(2.0), Value::Float(3.0)]).unwrap(),
            Value::Float(8.0)
        );
        assert_eq!(
            pow(&[Value::Int(2), Value::Int(10)]).unwrap(),
            Value::Float(1024.0)
        );
    }

    #[test]
    fn test_sin() {
        assert!(
            (sin(&[Value::Float(0.0)]).unwrap().as_float().unwrap() - 0.0).abs() < 1e-10
        );
        assert!(
            (sin(&[Value::Float(std::f64::consts::PI / 2.0)])
                .unwrap()
                .as_float()
                .unwrap()
                - 1.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_cos() {
        assert!(
            (cos(&[Value::Float(0.0)]).unwrap().as_float().unwrap() - 1.0).abs() < 1e-10
        );
        assert!(
            (cos(&[Value::Float(std::f64::consts::PI)])
                .unwrap()
                .as_float()
                .unwrap()
                + 1.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_tan() {
        assert!(
            (tan(&[Value::Float(0.0)]).unwrap().as_float().unwrap() - 0.0).abs() < 1e-10
        );
        assert!(
            (tan(&[Value::Float(std::f64::consts::PI / 4.0)])
                .unwrap()
                .as_float()
                .unwrap()
                - 1.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_ln() {
        assert!(
            (ln(&[Value::Float(1.0)]).unwrap().as_float().unwrap() - 0.0).abs() < 1e-10
        );
        assert!(
            (ln(&[Value::Float(std::f64::consts::E)])
                .unwrap()
                .as_float()
                .unwrap()
                - 1.0)
                .abs()
                < 1e-10
        );
        // Error on non-positive
        assert!(ln(&[Value::Float(-1.0)]).is_err());
    }

    #[test]
    fn test_log10() {
        assert!(
            (log10(&[Value::Float(1.0)]).unwrap().as_float().unwrap() - 0.0).abs()
                < 1e-10
        );
        assert!(
            (log10(&[Value::Float(100.0)]).unwrap().as_float().unwrap() - 2.0).abs()
                < 1e-10
        );
        // Error on non-positive
        assert!(log10(&[Value::Float(0.0)]).is_err());
    }

    #[test]
    fn test_exp() {
        assert!(
            (exp(&[Value::Float(0.0)]).unwrap().as_float().unwrap() - 1.0).abs() < 1e-10
        );
        assert!(
            (exp(&[Value::Float(1.0)]).unwrap().as_float().unwrap()
                - std::f64::consts::E)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_abs_null() {
        assert_eq!(abs(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_abs_bool() {
        assert_eq!(abs(&[Value::Bool(true)]).unwrap(), Value::Int(1));
        assert_eq!(abs(&[Value::Bool(false)]).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_abs_string() {
        assert_eq!(
            abs(&[Value::String("-42".to_string())]).unwrap(),
            Value::Int(42)
        );
        assert_eq!(
            abs(&[Value::String("-3.14".to_string())]).unwrap(),
            Value::Float(3.14)
        );
        // Invalid string should error
        assert!(abs(&[Value::String("abc".to_string())]).is_err());
    }

    #[test]
    fn test_round_string() {
        assert_eq!(
            round(&[Value::String("3.7".to_string())]).unwrap(),
            Value::Int(4)
        );
        // Invalid string should error
        assert!(round(&[Value::String("abc".to_string())]).is_err());
    }

    #[test]
    fn test_min_empty() {
        assert_eq!(min(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_min_no_args() {
        assert!(min(&[]).is_err());
    }

    #[test]
    fn test_min_mixed_types() {
        assert_eq!(
            min(&[
                Value::Int(3),
                Value::Float(1.5),
                Value::String("2".to_string())
            ])
            .unwrap(),
            Value::Float(1.5)
        );
    }

    #[test]
    fn test_min_with_null_and_list() {
        // Null should be skipped, List should be skipped
        assert_eq!(
            min(&[
                Value::Null,
                Value::Int(5),
                Value::List(vec![]),
                Value::Int(3)
            ])
            .unwrap(),
            Value::Int(3)
        );
    }

    #[test]
    fn test_max_empty() {
        assert_eq!(max(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_max_no_args() {
        assert!(max(&[]).is_err());
    }

    #[test]
    fn test_int_string_float() {
        // String containing float should be parsed and truncated
        assert_eq!(
            int(&[Value::String("3.14".to_string())]).unwrap(),
            Value::Int(3)
        );
    }

    #[test]
    fn test_int_invalid_string() {
        assert!(int(&[Value::String("abc".to_string())]).is_err());
    }

    #[test]
    fn test_int_bool() {
        assert_eq!(int(&[Value::Bool(true)]).unwrap(), Value::Int(1));
        assert_eq!(int(&[Value::Bool(false)]).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_float_invalid_string() {
        assert!(float(&[Value::String("abc".to_string())]).is_err());
    }

    #[test]
    fn test_float_bool() {
        assert_eq!(float(&[Value::Bool(true)]).unwrap(), Value::Float(1.0));
        assert_eq!(float(&[Value::Bool(false)]).unwrap(), Value::Float(0.0));
    }

    #[test]
    fn test_ceil_int() {
        // Int should pass through unchanged
        assert_eq!(ceil(&[Value::Int(5)]).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_ceil_string() {
        assert_eq!(
            ceil(&[Value::String("3.2".to_string())]).unwrap(),
            Value::Int(4)
        );
        assert!(ceil(&[Value::String("abc".to_string())]).is_err());
    }

    #[test]
    fn test_floor_int() {
        // Int should pass through unchanged
        assert_eq!(floor(&[Value::Int(5)]).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_floor_string() {
        assert_eq!(
            floor(&[Value::String("3.7".to_string())]).unwrap(),
            Value::Int(3)
        );
        assert!(floor(&[Value::String("abc".to_string())]).is_err());
    }

    #[test]
    fn test_sqrt_zero() {
        assert_eq!(sqrt(&[Value::Float(0.0)]).unwrap(), Value::Float(0.0));
    }

    #[test]
    fn test_sqrt_string() {
        assert_eq!(
            sqrt(&[Value::String("16".to_string())]).unwrap(),
            Value::Float(4.0)
        );
        assert!(sqrt(&[Value::String("abc".to_string())]).is_err());
    }

    #[test]
    fn test_sqrt_null() {
        assert_eq!(sqrt(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_pow_null() {
        // If either arg is Null, return Null
        assert_eq!(pow(&[Value::Null, Value::Int(2)]).unwrap(), Value::Null);
        assert_eq!(pow(&[Value::Int(2), Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_pow_string() {
        assert_eq!(
            pow(&[
                Value::String("2".to_string()),
                Value::String("3".to_string())
            ])
            .unwrap(),
            Value::Float(8.0)
        );
    }

    #[test]
    fn test_trigonometric_edge_cases() {
        // sin(π) ≈ 0
        assert!(
            (sin(&[Value::Float(std::f64::consts::PI)])
                .unwrap()
                .as_float()
                .unwrap()
                - 0.0)
                .abs()
                < 1e-10
        );
        // cos(π/2) ≈ 0
        assert!(
            (cos(&[Value::Float(std::f64::consts::PI / 2.0)])
                .unwrap()
                .as_float()
                .unwrap()
                - 0.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_ln_zero() {
        // ln(0) should error
        assert!(ln(&[Value::Float(0.0)]).is_err());
    }

    #[test]
    fn test_log10_negative() {
        assert!(log10(&[Value::Float(-1.0)]).is_err());
    }

    #[test]
    fn test_exp_zero() {
        assert_eq!(exp(&[Value::Float(0.0)]).unwrap(), Value::Float(1.0));
    }

    #[test]
    fn test_type_errors() {
        // Test that List, DateTime, Lambda produce type errors
        assert!(abs(&[Value::List(vec![])]).is_err());
        assert!(round(&[Value::List(vec![])]).is_err());
        assert!(int(&[Value::List(vec![])]).is_err());
        assert!(float(&[Value::List(vec![])]).is_err());
        assert!(ceil(&[Value::List(vec![])]).is_err());
        assert!(floor(&[Value::List(vec![])]).is_err());
        assert!(sqrt(&[Value::List(vec![])]).is_err());
    }

    // Additional tests for abs
    #[test]
    fn test_abs_positive() {
        assert_eq!(abs(&[Value::Int(5)]).unwrap(), Value::Int(5));
        assert_eq!(abs(&[Value::Float(3.14)]).unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_abs_zero() {
        assert_eq!(abs(&[Value::Int(0)]).unwrap(), Value::Int(0));
        assert_eq!(abs(&[Value::Float(0.0)]).unwrap(), Value::Float(0.0));
    }

    #[test]
    fn test_abs_negative_int() {
        assert_eq!(abs(&[Value::Int(-100)]).unwrap(), Value::Int(100));
        // Note: i64::MIN cannot be negated (would overflow), skipping this edge case
    }

    // Additional tests for round
    #[test]
    fn test_round_exact() {
        assert_eq!(round(&[Value::Float(5.0)]).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_round_half() {
        assert_eq!(round(&[Value::Float(2.5)]).unwrap(), Value::Int(3));
        assert_eq!(round(&[Value::Float(3.5)]).unwrap(), Value::Int(4));
    }

    #[test]
    fn test_round_negative() {
        assert_eq!(round(&[Value::Float(-2.3)]).unwrap(), Value::Int(-2));
        assert_eq!(round(&[Value::Float(-2.7)]).unwrap(), Value::Int(-3));
    }

    #[test]
    fn test_round_null() {
        assert_eq!(round(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_round_bool() {
        assert_eq!(round(&[Value::Bool(true)]).unwrap(), Value::Int(1));
        assert_eq!(round(&[Value::Bool(false)]).unwrap(), Value::Int(0));
    }

    // Additional tests for min/max
    #[test]
    fn test_min_single_value() {
        assert_eq!(min(&[Value::Int(42)]).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_min_all_null() {
        assert_eq!(min(&[Value::Null, Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_min_bool_values() {
        assert_eq!(
            min(&[Value::Bool(true), Value::Bool(false)]).unwrap(),
            Value::Int(0)
        );
    }

    #[test]
    fn test_min_string_values() {
        assert_eq!(
            min(&[
                Value::String("10".to_string()),
                Value::String("5".to_string())
            ])
            .unwrap(),
            Value::Int(5)
        );
    }

    #[test]
    fn test_max_single_value() {
        assert_eq!(max(&[Value::Int(42)]).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_max_all_null() {
        assert_eq!(max(&[Value::Null, Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_max_bool_values() {
        assert_eq!(
            max(&[Value::Bool(true), Value::Bool(false)]).unwrap(),
            Value::Int(1)
        );
    }

    #[test]
    fn test_max_string_values() {
        assert_eq!(
            max(&[
                Value::String("10".to_string()),
                Value::String("5".to_string())
            ])
            .unwrap(),
            Value::Int(10)
        );
    }

    // Additional tests for int
    #[test]
    fn test_int_null() {
        assert_eq!(int(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_int_negative_float() {
        assert_eq!(int(&[Value::Float(-3.7)]).unwrap(), Value::Int(-3));
    }

    #[test]
    fn test_int_negative_string() {
        assert_eq!(
            int(&[Value::String("-42".to_string())]).unwrap(),
            Value::Int(-42)
        );
    }

    #[test]
    fn test_int_large_number() {
        assert_eq!(
            int(&[Value::Int(9007199254740992i64)]).unwrap(),
            Value::Int(9007199254740992i64)
        );
    }

    // Additional tests for float
    #[test]
    fn test_float_null() {
        assert_eq!(float(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_float_negative() {
        assert_eq!(float(&[Value::Int(-42)]).unwrap(), Value::Float(-42.0));
    }

    #[test]
    fn test_float_negative_string() {
        assert_eq!(
            float(&[Value::String("-3.14".to_string())]).unwrap(),
            Value::Float(-3.14)
        );
    }

    #[test]
    fn test_float_scientific_notation() {
        assert_eq!(
            float(&[Value::String("1e10".to_string())]).unwrap(),
            Value::Float(1e10)
        );
    }

    // Additional tests for ceil
    #[test]
    fn test_ceil_null() {
        assert_eq!(ceil(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_ceil_bool() {
        assert_eq!(ceil(&[Value::Bool(true)]).unwrap(), Value::Int(1));
        assert_eq!(ceil(&[Value::Bool(false)]).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_ceil_negative() {
        assert_eq!(ceil(&[Value::Float(-3.2)]).unwrap(), Value::Int(-3));
        assert_eq!(ceil(&[Value::Float(-3.8)]).unwrap(), Value::Int(-3));
    }

    // Additional tests for floor
    #[test]
    fn test_floor_null() {
        assert_eq!(floor(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_floor_bool() {
        assert_eq!(floor(&[Value::Bool(true)]).unwrap(), Value::Int(1));
        assert_eq!(floor(&[Value::Bool(false)]).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_floor_negative() {
        assert_eq!(floor(&[Value::Float(-3.2)]).unwrap(), Value::Int(-4));
        assert_eq!(floor(&[Value::Float(-3.8)]).unwrap(), Value::Int(-4));
    }

    // Additional tests for sqrt
    #[test]
    fn test_sqrt_int() {
        assert_eq!(sqrt(&[Value::Int(16)]).unwrap(), Value::Float(4.0));
    }

    #[test]
    fn test_sqrt_bool() {
        assert_eq!(sqrt(&[Value::Bool(true)]).unwrap(), Value::Float(1.0));
        assert_eq!(sqrt(&[Value::Bool(false)]).unwrap(), Value::Float(0.0));
    }

    #[test]
    fn test_sqrt_one() {
        assert_eq!(sqrt(&[Value::Float(1.0)]).unwrap(), Value::Float(1.0));
    }

    // Additional tests for pow
    #[test]
    fn test_pow_zero_exponent() {
        assert_eq!(
            pow(&[Value::Int(5), Value::Int(0)]).unwrap(),
            Value::Float(1.0)
        );
    }

    #[test]
    fn test_pow_zero_base() {
        assert_eq!(
            pow(&[Value::Int(0), Value::Int(5)]).unwrap(),
            Value::Float(0.0)
        );
    }

    #[test]
    fn test_pow_negative_exponent() {
        assert_eq!(
            pow(&[Value::Int(2), Value::Int(-1)]).unwrap(),
            Value::Float(0.5)
        );
    }

    #[test]
    fn test_pow_fractional() {
        assert!(
            (pow(&[Value::Float(4.0), Value::Float(0.5)])
                .unwrap()
                .as_float()
                .unwrap()
                - 2.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_pow_bool() {
        assert_eq!(
            pow(&[Value::Bool(true), Value::Bool(true)]).unwrap(),
            Value::Float(1.0)
        );
    }

    // Additional tests for trigonometric functions
    #[test]
    fn test_sin_null() {
        assert_eq!(sin(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_sin_bool() {
        assert!(
            (sin(&[Value::Bool(true)]).unwrap().as_float().unwrap() - 1.0f64.sin())
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_cos_null() {
        assert_eq!(cos(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_cos_bool() {
        assert!(
            (cos(&[Value::Bool(true)]).unwrap().as_float().unwrap() - 1.0f64.cos())
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_tan_null() {
        assert_eq!(tan(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_tan_bool() {
        assert!(
            (tan(&[Value::Bool(true)]).unwrap().as_float().unwrap() - 1.0f64.tan())
                .abs()
                < 1e-10
        );
    }

    // Additional tests for logarithmic functions
    #[test]
    fn test_ln_one() {
        assert!(
            (ln(&[Value::Float(1.0)]).unwrap().as_float().unwrap() - 0.0).abs() < 1e-10
        );
    }

    #[test]
    fn test_ln_null() {
        assert_eq!(ln(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_ln_bool() {
        assert!(
            (ln(&[Value::Bool(true)]).unwrap().as_float().unwrap() - 0.0).abs() < 1e-10
        );
    }

    #[test]
    fn test_log10_one() {
        assert!(
            (log10(&[Value::Float(1.0)]).unwrap().as_float().unwrap() - 0.0).abs()
                < 1e-10
        );
    }

    #[test]
    fn test_log10_null() {
        assert_eq!(log10(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_log10_bool() {
        assert!(
            (log10(&[Value::Bool(true)]).unwrap().as_float().unwrap() - 0.0).abs()
                < 1e-10
        );
    }

    #[test]
    fn test_log10_10() {
        assert!(
            (log10(&[Value::Float(10.0)]).unwrap().as_float().unwrap() - 1.0).abs()
                < 1e-10
        );
    }

    // Additional tests for exp
    #[test]
    fn test_exp_null() {
        assert_eq!(exp(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_exp_bool() {
        assert!(
            (exp(&[Value::Bool(true)]).unwrap().as_float().unwrap()
                - std::f64::consts::E)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn test_exp_negative() {
        assert!(
            (exp(&[Value::Float(-1.0)]).unwrap().as_float().unwrap()
                - 1.0 / std::f64::consts::E)
                .abs()
                < 1e-10
        );
    }

    // Test type errors for DateTime and Lambda
    #[test]
    fn test_datetime_type_errors() {
        use chrono::Utc;

        let dt = Value::DateTime(Utc::now());
        assert!(abs(&[dt.clone()]).is_err());
        assert!(round(&[dt.clone()]).is_err());
        assert!(int(&[dt.clone()]).is_err());
        assert!(float(&[dt.clone()]).is_err());
        assert!(ceil(&[dt.clone()]).is_err());
        assert!(floor(&[dt.clone()]).is_err());
        assert!(sqrt(&[dt.clone()]).is_err());
        assert!(pow(&[dt.clone(), Value::Int(2)]).is_err());
        assert!(sin(&[dt.clone()]).is_err());
        assert!(cos(&[dt.clone()]).is_err());
        assert!(tan(&[dt.clone()]).is_err());
        assert!(ln(&[dt.clone()]).is_err());
        assert!(log10(&[dt.clone()]).is_err());
        assert!(exp(&[dt.clone()]).is_err());
    }

    #[test]
    fn test_lambda_type_errors() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });
        assert!(abs(&[lambda.clone()]).is_err());
        assert!(round(&[lambda.clone()]).is_err());
        assert!(int(&[lambda.clone()]).is_err());
        assert!(float(&[lambda.clone()]).is_err());
        assert!(ceil(&[lambda.clone()]).is_err());
        assert!(floor(&[lambda.clone()]).is_err());
        assert!(sqrt(&[lambda.clone()]).is_err());
        assert!(pow(&[lambda.clone(), Value::Int(2)]).is_err());
        assert!(sin(&[lambda.clone()]).is_err());
        assert!(cos(&[lambda.clone()]).is_err());
        assert!(tan(&[lambda.clone()]).is_err());
        assert!(ln(&[lambda.clone()]).is_err());
        assert!(log10(&[lambda.clone()]).is_err());
        assert!(exp(&[lambda.clone()]).is_err());
    }

    // Test edge cases with special float values
    #[test]
    fn test_special_float_values() {
        // sqrt of infinity
        assert!(sqrt(&[Value::Float(f64::INFINITY)])
            .unwrap()
            .as_float()
            .unwrap()
            .is_infinite());
        // sqrt of NaN
        assert!(sqrt(&[Value::Float(f64::NAN)])
            .unwrap()
            .as_float()
            .unwrap()
            .is_nan());
        // exp of infinity
        assert!(exp(&[Value::Float(f64::INFINITY)])
            .unwrap()
            .as_float()
            .unwrap()
            .is_infinite());
        // exp of large negative
        assert_eq!(
            exp(&[Value::Float(-1000.0)]).unwrap().as_float().unwrap(),
            0.0
        );
    }

    // Test max/min with DateTime and Lambda (should be skipped)
    #[test]
    fn test_min_max_skips_datetime_lambda() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;
        use chrono::Utc;

        let dt = Value::DateTime(Utc::now());
        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });

        // DateTime and Lambda should be skipped
        assert_eq!(
            min(&[dt.clone(), Value::Int(5), lambda.clone()]).unwrap(),
            Value::Int(5)
        );
        assert_eq!(
            max(&[dt.clone(), Value::Int(5), lambda.clone()]).unwrap(),
            Value::Int(5)
        );
    }

    // Additional tests to cover error handling branches

    #[test]
    fn test_min_with_list() {
        // List should be skipped in min
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(min(&[list, Value::Int(5)]).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_max_with_list() {
        // List should be skipped in max
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(max(&[list, Value::Int(5)]).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_int_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = int(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_float_invalid_string_error() {
        let result = float(&[Value::String("not-a-number".to_string())]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot convert"));
    }

    #[test]
    fn test_pow_with_list_base_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = pow(&[list, Value::Int(2)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_pow_with_datetime_base_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = pow(&[dt, Value::Int(2)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_pow_with_lambda_base_error() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });
        let result = pow(&[lambda, Value::Int(2)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert lambda"));
    }

    #[test]
    fn test_pow_with_list_exp_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = pow(&[Value::Int(2), list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_pow_with_datetime_exp_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = pow(&[Value::Int(2), dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_sin_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = sin(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_sin_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = sin(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_sin_with_lambda_error() {
        use crate::libs::expr::parser::ast::Expr;
        use crate::libs::expr::runtime::value::LambdaValue;
        use ahash::HashMap;

        let lambda = Value::Lambda(LambdaValue {
            captured_vars: HashMap::new(),
            params: vec!["x".to_string()],
            body: Expr::LambdaParam("x".to_string()),
        });
        let result = sin(&[lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert lambda"));
    }

    #[test]
    fn test_cos_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = cos(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_cos_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = cos(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_tan_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = tan(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_tan_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = tan(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_ln_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = ln(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_ln_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = ln(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_log10_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = log10(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_log10_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = log10(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_exp_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = exp(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_exp_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = exp(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_sqrt_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = sqrt(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_sqrt_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = sqrt(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_int_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = int(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_float_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = float(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_float_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = float(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_ceil_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = ceil(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_ceil_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = ceil(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_floor_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = floor(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_floor_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = floor(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_abs_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = abs(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_abs_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = abs(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    #[test]
    fn test_round_with_list_error() {
        let list = Value::List(vec![Value::Int(1)]);
        let result = round(&[list]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert list"));
    }

    #[test]
    fn test_round_with_datetime_error() {
        use chrono::Utc;
        let dt = Value::DateTime(Utc::now());
        let result = round(&[dt]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot convert datetime"));
    }

    // Tests moved from src/libs/expr/tests/functions.rs
    #[test]
    fn test_abs_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["-5".to_string()];
        assert_eq!(eval_expr("abs(@1)", &row, None).unwrap().to_string(), "5");

        let row: Vec<String> = vec!["-3.14".to_string()];
        assert_eq!(
            eval_expr("abs(@1)", &row, None).unwrap().to_string(),
            "3.14"
        );
    }

    #[test]
    fn test_round_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec!["3.7".to_string()];
        assert_eq!(eval_expr("round(@1)", &row, None).unwrap().to_string(), "4");

        let row: Vec<String> = vec!["3.2".to_string()];
        assert_eq!(eval_expr("round(@1)", &row, None).unwrap().to_string(), "3");
    }

    #[test]
    fn test_min_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec![];
        assert_eq!(
            eval_expr("min(3, 1, 2)", &row, None).unwrap().to_string(),
            "1"
        );
        assert_eq!(
            eval_expr("min(10, 5)", &row, None).unwrap().to_string(),
            "5"
        );
    }

    #[test]
    fn test_max_integration() {
        use crate::libs::expr::eval_expr;
        let row: Vec<String> = vec![];
        assert_eq!(
            eval_expr("max(3, 1, 2)", &row, None).unwrap().to_string(),
            "3"
        );
        assert_eq!(
            eval_expr("max(10, 5)", &row, None).unwrap().to_string(),
            "10"
        );
    }
}
