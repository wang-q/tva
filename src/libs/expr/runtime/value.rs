use ahash::HashMap;
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

    /// Modulo operation
    pub fn modulo(
        &self,
        rhs: &Value,
    ) -> Result<Value, crate::libs::expr::runtime::EvalError> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(crate::libs::expr::runtime::EvalError::DivisionByZero)
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            _ => {
                let a = self.as_f64().ok_or_else(|| {
                    crate::libs::expr::runtime::EvalError::TypeError(
                        "Modulo requires numbers".to_string(),
                    )
                })?;
                let b = rhs.as_f64().ok_or_else(|| {
                    crate::libs::expr::runtime::EvalError::TypeError(
                        "Modulo requires numbers".to_string(),
                    )
                })?;
                if b == 0.0 {
                    Err(crate::libs::expr::runtime::EvalError::DivisionByZero)
                } else {
                    Ok(Value::Float(a % b))
                }
            }
        }
    }

    /// String concatenation
    pub fn concat(
        &self,
        rhs: &Value,
    ) -> Result<Value, crate::libs::expr::runtime::EvalError> {
        let left_str = self.to_string();
        let right_str = rhs.to_string();
        Ok(Value::String(left_str + &right_str))
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
    use ahash::HashMapExt;

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

    #[test]
    fn test_is_numeric() {
        assert_eq!(Value::Int(42).is_numeric(), true);
        assert_eq!(Value::Float(3.14).is_numeric(), true);
        assert_eq!(Value::Null.is_numeric(), false);
        assert_eq!(Value::Bool(true).is_numeric(), false);
        assert_eq!(Value::String("123".to_string()).is_numeric(), false);
        assert_eq!(Value::List(vec![]).is_numeric(), false);
        assert_eq!(Value::DateTime(chrono::Utc::now()).is_numeric(), false);
        assert_eq!(
            Value::Lambda(LambdaValue {
                params: vec![],
                body: crate::libs::expr::parser::ast::Expr::Int(1),
                captured_vars: HashMap::new(),
            })
            .is_numeric(),
            false
        );
    }

    #[test]
    fn test_is_null() {
        assert_eq!(Value::Null.is_null(), true);
        assert_eq!(Value::Int(42).is_null(), false);
        assert_eq!(Value::Bool(false).is_null(), false);
        assert_eq!(Value::String("".to_string()).is_null(), false);
    }

    #[test]
    fn test_type_name() {
        assert_eq!(Value::Null.type_name(), "null");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::Int(42).type_name(), "int");
        assert_eq!(Value::Float(3.14).type_name(), "float");
        assert_eq!(Value::String("hello".to_string()).type_name(), "string");
        assert_eq!(Value::List(vec![]).type_name(), "list");
        assert_eq!(Value::DateTime(chrono::Utc::now()).type_name(), "datetime");
        assert_eq!(
            Value::Lambda(LambdaValue {
                params: vec![],
                body: crate::libs::expr::parser::ast::Expr::Int(1),
                captured_vars: HashMap::new(),
            })
            .type_name(),
            "lambda"
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Value::Null.to_string(), "null");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
        assert_eq!(Value::String("hello".to_string()).to_string(), "hello");
        assert_eq!(
            Value::List(vec![Value::Int(1), Value::Int(2)]).to_string(),
            "[Int(1), Int(2)]"
        );
        let dt = chrono::Utc::now();
        assert_eq!(Value::DateTime(dt).to_string(), dt.to_rfc3339());
        assert_eq!(
            Value::Lambda(LambdaValue {
                params: vec![],
                body: crate::libs::expr::parser::ast::Expr::Int(1),
                captured_vars: HashMap::new(),
            })
            .to_string(),
            "<lambda>"
        );
    }

    #[test]
    fn test_as_string() {
        assert_eq!(Value::String("hello".to_string()).as_string(), "hello");
        assert_eq!(Value::Int(42).as_string(), "42");
        assert_eq!(Value::Null.as_string(), "null");
        assert_eq!(Value::Bool(true).as_string(), "true");
    }

    #[test]
    fn test_as_int() {
        // Int
        assert_eq!(Value::Int(42).as_int(), Some(42));
        // Float (truncates)
        assert_eq!(Value::Float(3.7).as_int(), Some(3));
        // String that parses
        assert_eq!(Value::String("123".to_string()).as_int(), Some(123));
        // String that doesn't parse
        assert_eq!(Value::String("abc".to_string()).as_int(), None);
        // Bool
        assert_eq!(Value::Bool(true).as_int(), Some(1));
        assert_eq!(Value::Bool(false).as_int(), Some(0));
        // Other types return None
        assert_eq!(Value::Null.as_int(), None);
        assert_eq!(Value::List(vec![]).as_int(), None);
    }

    #[test]
    fn test_as_f64_non_numeric() {
        assert_eq!(Value::Null.as_f64(), None);
        assert_eq!(Value::Bool(true).as_f64(), None);
        assert_eq!(Value::String("hello".to_string()).as_f64(), None);
        assert_eq!(Value::List(vec![]).as_f64(), None);
        assert_eq!(Value::DateTime(chrono::Utc::now()).as_f64(), None);
    }

    #[test]
    fn test_as_float() {
        assert_eq!(Value::Float(3.14).as_float(), Some(3.14));
        assert_eq!(Value::Int(42).as_float(), None);
        assert_eq!(Value::Null.as_float(), None);
    }

    #[test]
    fn test_display_trait() {
        use std::fmt::Write;
        let mut buf = String::new();
        write!(&mut buf, "{}", Value::Int(42)).unwrap();
        assert_eq!(buf, "42");

        buf.clear();
        write!(&mut buf, "{}", Value::String("hello".to_string())).unwrap();
        assert_eq!(buf, "hello");
    }

    #[test]
    fn test_as_bool_with_list() {
        // Empty list is falsy
        assert_eq!(Value::List(vec![]).as_bool(), false);
        // Non-empty list is truthy
        assert_eq!(Value::List(vec![Value::Int(1)]).as_bool(), true);
    }

    #[test]
    fn test_as_bool_with_datetime() {
        use chrono::Utc;
        // DateTime is always truthy
        assert_eq!(Value::DateTime(Utc::now()).as_bool(), true);
    }

    #[test]
    fn test_as_bool_with_lambda() {
        // Lambda is always truthy
        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: crate::libs::expr::parser::ast::Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        assert_eq!(lambda.as_bool(), true);
    }

    #[test]
    fn test_comparison_with_non_numeric() {
        // lt with non-numeric should return None
        assert_eq!(Value::String("a".to_string()).lt(&Value::Int(1)), None);
        assert_eq!(Value::Null.lt(&Value::Int(1)), None);

        // le with non-numeric should return None
        assert_eq!(Value::String("a".to_string()).le(&Value::Int(1)), None);

        // gt with non-numeric should return None
        assert_eq!(Value::List(vec![]).gt(&Value::Int(1)), None);

        // ge with non-numeric should return None
        assert_eq!(Value::Bool(true).ge(&Value::Int(1)), None);
    }

    #[test]
    fn test_arithmetic_with_non_numeric() {
        // Add with non-numeric should return None
        assert_eq!(Value::String("a".to_string()) + Value::Int(1), None);

        // Sub with non-numeric should return None
        assert_eq!(Value::Null - Value::Int(1), None);

        // Mul with non-numeric should return None
        assert_eq!(Value::Bool(true) * Value::Int(1), None);

        // Div with non-numeric should return None
        assert_eq!(Value::List(vec![]) / Value::Int(1), None);
    }

    #[test]
    fn test_division_by_zero() {
        assert_eq!(Value::Int(10) / Value::Int(0), None);
        assert_eq!(Value::Float(10.0) / Value::Float(0.0), None);
    }

    #[test]
    fn test_modulo_by_zero() {
        assert_eq!(Value::Int(10) % Value::Int(0), None);
        assert_eq!(Value::Float(10.0) % Value::Float(0.0), None);
    }

    #[test]
    fn test_modulo_float() {
        assert_eq!(
            Value::Float(10.5) % Value::Float(3.0),
            Some(Value::Float(1.5))
        );
    }

    #[test]
    fn test_pow_with_non_numeric() {
        assert_eq!(Value::String("a".to_string()).pow(&Value::Int(2)), None);
        assert_eq!(Value::Int(2).pow(&Value::String("b".to_string())), None);
    }

    #[test]
    fn test_compare_datetime() {
        use chrono::Utc;
        let dt1 = chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let dt2 = chrono::DateTime::parse_from_rfc3339("2023-06-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(
            Value::DateTime(dt1).compare(&Value::DateTime(dt2)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::DateTime(dt2).compare(&Value::DateTime(dt1)),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(
            Value::DateTime(dt1).compare(&Value::DateTime(dt1)),
            Some(std::cmp::Ordering::Equal)
        );
    }

    #[test]
    fn test_compare_lambda_returns_none() {
        let lambda1 = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: crate::libs::expr::parser::ast::Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::new(),
        });
        let lambda2 = Value::Lambda(LambdaValue {
            params: vec!["y".to_string()],
            body: crate::libs::expr::parser::ast::Expr::LambdaParam("y".to_string()),
            captured_vars: HashMap::new(),
        });

        // Lambda comparison returns None (not comparable)
        assert_eq!(lambda1.compare(&lambda2), None);
        assert_eq!(lambda1.compare(&lambda1), None);
    }

    #[test]
    fn test_compare_mixed_types_with_datetime() {
        use chrono::Utc;
        // DateTime has higher priority than list
        assert_eq!(
            Value::List(vec![]).compare(&Value::DateTime(Utc::now())),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::DateTime(Utc::now()).compare(&Value::List(vec![])),
            Some(std::cmp::Ordering::Greater)
        );

        // Lambda has highest priority
        let lambda = Value::Lambda(LambdaValue {
            params: vec![],
            body: crate::libs::expr::parser::ast::Expr::Int(1),
            captured_vars: HashMap::new(),
        });
        assert_eq!(
            Value::DateTime(Utc::now()).compare(&lambda),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_compare_list_with_incomparable_elements() {
        // List with lambda elements (not comparable)
        let lambda = Value::Lambda(LambdaValue {
            params: vec![],
            body: crate::libs::expr::parser::ast::Expr::Int(1),
            captured_vars: HashMap::new(),
        });
        // Two lists with lambda elements at same position - should return None
        let list1 = Value::List(vec![lambda.clone()]);
        let list2 = Value::List(vec![lambda.clone()]);

        // Comparison should return None when elements are not comparable
        assert_eq!(list1.compare(&list2), None);
    }

    #[test]
    fn test_float_comparison_with_nan() {
        // Test float comparison with NaN
        let nan = Value::Float(f64::NAN);
        let num = Value::Float(1.0);

        // NaN comparisons return None
        assert_eq!(nan.compare(&num), None);
        assert_eq!(num.compare(&nan), None);
        assert_eq!(nan.compare(&nan), None);
    }

    #[test]
    fn test_ne_comparison() {
        let a = Value::Int(10);
        let b = Value::Int(5);
        let c = Value::Int(10);

        assert_eq!(a.ne(&b), Value::Bool(true));
        assert_eq!(a.ne(&c), Value::Bool(false));
    }

    #[test]
    fn test_eq_comparison() {
        let a = Value::Int(10);
        let b = Value::Int(5);
        let c = Value::Int(10);

        assert_eq!(a.eq(&b), Value::Bool(false));
        assert_eq!(a.eq(&c), Value::Bool(true));
    }
}
