use std::fmt;
use std::ops::{Add, Div, Mul, Rem, Sub};

/// Runtime value type for expression evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
}

impl Value {
    /// Convert to f64 for numeric operations
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Convert to bool for logical operations
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(0) => false,
            Value::Int(_) => true,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(list) => !list.is_empty(),
        }
    }

    /// Check if value is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Float(_) | Value::Int(_))
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::List(_) => "list",
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::List(list) => format!("{:?}", list),
        }
    }

    /// Convert to string (for string operations)
    pub fn as_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            v => v.to_string(),
        }
    }

    /// Convert to i64 (for numeric operations)
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            Value::String(s) => s.parse().ok(),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Power operation
    pub fn pow(&self, rhs: &Value) -> Option<Value> {
        let a = self.as_f64()?;
        let b = rhs.as_f64()?;
        Some(Value::Float(a.powf(b)))
    }

    /// Comparison operations
    pub fn eq(&self, rhs: &Value) -> Value {
        Value::Bool(self == rhs)
    }

    pub fn ne(&self, rhs: &Value) -> Value {
        Value::Bool(self != rhs)
    }

    pub fn lt(&self, rhs: &Value) -> Option<Value> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(a < b)),
            _ => {
                let a = self.as_f64()?;
                let b = rhs.as_f64()?;
                Some(Value::Bool(a < b))
            }
        }
    }

    pub fn le(&self, rhs: &Value) -> Option<Value> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(a <= b)),
            _ => {
                let a = self.as_f64()?;
                let b = rhs.as_f64()?;
                Some(Value::Bool(a <= b))
            }
        }
    }

    pub fn gt(&self, rhs: &Value) -> Option<Value> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(a > b)),
            _ => {
                let a = self.as_f64()?;
                let b = rhs.as_f64()?;
                Some(Value::Bool(a > b))
            }
        }
    }

    pub fn ge(&self, rhs: &Value) -> Option<Value> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(a >= b)),
            _ => {
                let a = self.as_f64()?;
                let b = rhs.as_f64()?;
                Some(Value::Bool(a >= b))
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// Arithmetic operations
impl Add for Value {
    type Output = Option<Value>;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(a + b)),
            _ => {
                let a = self.as_f64()?;
                let b = rhs.as_f64()?;
                Some(Value::Float(a + b))
            }
        }
    }
}

impl Sub for Value {
    type Output = Option<Value>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(a - b)),
            _ => {
                let a = self.as_f64()?;
                let b = rhs.as_f64()?;
                Some(Value::Float(a - b))
            }
        }
    }
}

impl Mul for Value {
    type Output = Option<Value>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(a * b)),
            _ => {
                let a = self.as_f64()?;
                let b = rhs.as_f64()?;
                Some(Value::Float(a * b))
            }
        }
    }
}

impl Div for Value {
    type Output = Option<Value>;

    fn div(self, rhs: Self) -> Self::Output {
        let a = self.as_f64()?;
        let b = rhs.as_f64()?;
        if b == 0.0 {
            return None;
        }
        Some(Value::Float(a / b))
    }
}

impl Rem for Value {
    type Output = Option<Value>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return None;
                }
                Some(Value::Int(a % b))
            }
            _ => {
                let a = self.as_f64()?;
                let b = rhs.as_f64()?;
                if b == 0.0 {
                    return None;
                }
                Some(Value::Float(a % b))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_arithmetic() {
        let a = Value::Float(10.0);
        let b = Value::Float(3.0);

        assert_eq!(a.clone() + b.clone(), Some(Value::Float(13.0)));
        assert_eq!(a.clone() - b.clone(), Some(Value::Float(7.0)));
        assert_eq!(a.clone() * b.clone(), Some(Value::Float(30.0)));
        assert_eq!(a.clone() / b.clone(), Some(Value::Float(10.0 / 3.0)));
    }

    #[test]
    fn test_value_int_arithmetic() {
        let a = Value::Int(10);
        let b = Value::Int(3);

        assert_eq!(a.clone() + b.clone(), Some(Value::Int(13)));
        assert_eq!(a.clone() - b.clone(), Some(Value::Int(7)));
        assert_eq!(a.clone() * b.clone(), Some(Value::Int(30)));
        // Division promotes to float
        assert_eq!(a.clone() / b.clone(), Some(Value::Float(10.0 / 3.0)));
    }

    #[test]
    fn test_value_mixed_arithmetic() {
        let a = Value::Int(10);
        let b = Value::Float(3.5);

        assert_eq!(a.clone() + b.clone(), Some(Value::Float(13.5)));
        assert_eq!(a.clone() - b.clone(), Some(Value::Float(6.5)));
        assert_eq!(a.clone() * b.clone(), Some(Value::Float(35.0)));
    }

    #[test]
    fn test_value_comparison() {
        let a = Value::Int(10);
        let b = Value::Int(5);

        assert_eq!(a.lt(&b), Some(Value::Bool(false)));
        assert_eq!(a.gt(&b), Some(Value::Bool(true)));
        assert_eq!(a.eq(&b), Value::Bool(false));
    }

    #[test]
    fn test_value_power() {
        let a = Value::Int(2);
        let b = Value::Int(3);

        assert_eq!(a.pow(&b), Some(Value::Float(8.0)));
    }

    #[test]
    fn test_value_modulo() {
        let a = Value::Int(10);
        let b = Value::Int(3);

        assert_eq!(a % b, Some(Value::Int(1)));
    }

    #[test]
    fn test_as_bool() {
        assert_eq!(Value::Null.as_bool(), false);
        assert_eq!(Value::Bool(true).as_bool(), true);
        assert_eq!(Value::Bool(false).as_bool(), false);
        assert_eq!(Value::Int(0).as_bool(), false);
        assert_eq!(Value::Int(5).as_bool(), true);
        assert_eq!(Value::Float(0.0).as_bool(), false);
        assert_eq!(Value::Float(1.5).as_bool(), true);
        assert_eq!(Value::String("".to_string()).as_bool(), false);
        assert_eq!(Value::String("hello".to_string()).as_bool(), true);
    }
}
