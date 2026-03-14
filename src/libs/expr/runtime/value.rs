use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Div, Mul, Rem, Sub};

/// Lambda function value with captured variables
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaValue {
    pub params: Vec<String>,
    pub body: crate::libs::expr::parser::ast::Expr,
    /// Captured variables from the enclosing scope
    pub captured_vars: HashMap<String, Value>,
}

/// Runtime value type for expression evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    DateTime(chrono::DateTime<chrono::Utc>),
    Lambda(LambdaValue),
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

    /// Get float value (for test assertions)
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
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
            Value::DateTime(_) => true,
            Value::Lambda(_) => true,
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
            Value::DateTime(_) => "datetime",
            Value::Lambda(_) => "lambda",
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
            Value::DateTime(dt) => dt.to_rfc3339(),
            Value::Lambda(_) => "<lambda>".to_string(),
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

    /// Compare two values for ordering (used by sort_by)
    /// Returns Ordering::Less if self < other, etc.
    /// Ordering: null < bool < int/float < string < list < others
    pub fn compare(&self, other: &Value) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;

        // Get type priority (lower = comes first)
        let type_priority = |v: &Value| match v {
            Value::Null => 0,
            Value::Bool(_) => 1,
            Value::Int(_) | Value::Float(_) => 2,
            Value::String(_) => 3,
            Value::List(_) => 4,
            Value::DateTime(_) => 5,
            Value::Lambda(_) => 6,
        };

        let self_prio = type_priority(self);
        let other_prio = type_priority(other);

        if self_prio != other_prio {
            return Some(self_prio.cmp(&other_prio));
        }

        // Same type, compare values
        match (self, other) {
            (Value::Null, Value::Null) => Some(Ordering::Equal),
            (Value::Bool(a), Value::Bool(b)) => Some(a.cmp(b)),
            (Value::Int(a), Value::Int(b)) => Some(a.cmp(b)),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
            (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
            (Value::List(a), Value::List(b)) => {
                // Lexicographical comparison
                let min_len = a.len().min(b.len());
                for i in 0..min_len {
                    match a[i].compare(&b[i])? {
                        Ordering::Equal => continue,
                        other => return Some(other),
                    }
                }
                Some(a.len().cmp(&b.len()))
            }
            (Value::DateTime(a), Value::DateTime(b)) => Some(a.cmp(b)),
            // Lambda and other types are not comparable
            _ => None,
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

    #[test]
    fn test_value_compare_null() {
        assert_eq!(
            Value::Null.compare(&Value::Null),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            Value::Null.compare(&Value::Int(1)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Int(1).compare(&Value::Null),
            Some(std::cmp::Ordering::Greater)
        );
    }

    #[test]
    fn test_value_compare_bool() {
        assert_eq!(
            Value::Bool(false).compare(&Value::Bool(true)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Bool(true).compare(&Value::Bool(true)),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            Value::Bool(true).compare(&Value::Bool(false)),
            Some(std::cmp::Ordering::Greater)
        );
    }

    #[test]
    fn test_value_compare_int() {
        assert_eq!(
            Value::Int(1).compare(&Value::Int(2)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Int(5).compare(&Value::Int(5)),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            Value::Int(10).compare(&Value::Int(3)),
            Some(std::cmp::Ordering::Greater)
        );
    }

    #[test]
    fn test_value_compare_float() {
        assert_eq!(
            Value::Float(1.5).compare(&Value::Float(2.5)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Float(3.0).compare(&Value::Float(3.0)),
            Some(std::cmp::Ordering::Equal)
        );
    }

    #[test]
    fn test_value_compare_int_float() {
        assert_eq!(
            Value::Int(1).compare(&Value::Float(2.0)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Int(5).compare(&Value::Float(5.0)),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            Value::Float(1.5).compare(&Value::Int(2)),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_value_compare_string() {
        assert_eq!(
            Value::String("apple".to_string())
                .compare(&Value::String("banana".to_string())),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::String("zebra".to_string())
                .compare(&Value::String("apple".to_string())),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(
            Value::String("same".to_string())
                .compare(&Value::String("same".to_string())),
            Some(std::cmp::Ordering::Equal)
        );
    }

    #[test]
    fn test_value_compare_list() {
        // Empty list comparison
        let empty: Vec<Value> = vec![];
        let list1 = Value::List(vec![Value::Int(1)]);
        let list_empty = Value::List(empty.clone());
        assert_eq!(
            list_empty.compare(&list_empty),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(list_empty.compare(&list1), Some(std::cmp::Ordering::Less));
        assert_eq!(
            list1.compare(&list_empty),
            Some(std::cmp::Ordering::Greater)
        );

        // List with same elements
        let a = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let b = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(a.compare(&b), Some(std::cmp::Ordering::Equal));

        // List with different elements
        let c = Value::List(vec![Value::Int(1), Value::Int(3)]);
        assert_eq!(a.compare(&c), Some(std::cmp::Ordering::Less));

        // List with different lengths
        let d = Value::List(vec![Value::Int(1)]);
        assert_eq!(d.compare(&a), Some(std::cmp::Ordering::Less));
        assert_eq!(a.compare(&d), Some(std::cmp::Ordering::Greater));

        // Nested list comparison
        let nested1 = Value::List(vec![
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::Int(3),
        ]);
        let nested2 = Value::List(vec![
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::Int(4),
        ]);
        assert_eq!(nested1.compare(&nested2), Some(std::cmp::Ordering::Less));
    }

    #[test]
    fn test_value_compare_mixed_types() {
        // Type priority: null < bool < int/float < string < list
        assert_eq!(
            Value::Null.compare(&Value::Bool(true)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Bool(false).compare(&Value::Int(0)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Int(100).compare(&Value::String("a".to_string())),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::String("z".to_string()).compare(&Value::List(vec![])),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_value_compare_list_lexicographical() {
        // Test lexicographical ordering with composite keys like [r.nth(0), r.nth(1)]
        let a = Value::List(vec![Value::Int(1), Value::String("a".to_string())]);
        let b = Value::List(vec![Value::Int(1), Value::String("c".to_string())]);
        let c = Value::List(vec![Value::Int(2), Value::String("b".to_string())]);

        assert_eq!(a.compare(&b), Some(std::cmp::Ordering::Less)); // [1, "a"] < [1, "c"]
        assert_eq!(a.compare(&c), Some(std::cmp::Ordering::Less)); // [1, "a"] < [2, "b"]
        assert_eq!(b.compare(&c), Some(std::cmp::Ordering::Less)); // [1, "c"] < [2, "b"]
    }
}
