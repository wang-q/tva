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
    fn test_min_empty() {
        assert_eq!(min(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_max_empty() {
        assert_eq!(max(&[Value::Null]).unwrap(), Value::Null);
    }
}
