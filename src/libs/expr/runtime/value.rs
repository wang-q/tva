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
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl}"),
            Value::String(s) => write!(f, "{s}"),
            Value::List(list) => write!(f, "{list:?}"),
            Value::DateTime(dt) => write!(f, "{}", dt.to_rfc3339()),
            Value::Lambda(_) => write!(f, "<lambda>"),
        }
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
    use crate::libs::expr::EvalError;
    use test_case::test_case;

    #[test_case(Value::Float(10.0), Value::Float(3.0), Value::Float(13.0) ; "float_add")]
    #[test_case(Value::Int(10), Value::Int(3), Value::Int(13) ; "int_add")]
    #[test_case(Value::Int(10), Value::Float(3.5), Value::Float(13.5) ; "mixed_add")]
    fn test_value_add(left: Value, right: Value, expected: Value) {
        assert_eq!(left + right, Some(expected));
    }

    #[test_case(Value::Float(10.0), Value::Float(3.0), Value::Float(7.0) ; "float_sub")]
    #[test_case(Value::Int(10), Value::Int(3), Value::Int(7) ; "int_sub")]
    #[test_case(Value::Int(10), Value::Float(3.5), Value::Float(6.5) ; "mixed_sub")]
    fn test_value_sub(left: Value, right: Value, expected: Value) {
        assert_eq!(left - right, Some(expected));
    }

    #[test_case(Value::Float(10.0), Value::Float(3.0), Value::Float(30.0) ; "float_mul")]
    #[test_case(Value::Int(10), Value::Int(3), Value::Int(30) ; "int_mul")]
    #[test_case(Value::Int(10), Value::Float(3.5), Value::Float(35.0) ; "mixed_mul")]
    fn test_value_mul(left: Value, right: Value, expected: Value) {
        assert_eq!(left * right, Some(expected));
    }

    #[test]
    fn test_value_div() {
        assert_eq!(
            Value::Float(10.0) / Value::Float(3.0),
            Some(Value::Float(10.0 / 3.0))
        );
        assert_eq!(
            Value::Int(10) / Value::Int(3),
            Some(Value::Float(10.0 / 3.0))
        );
    }

    #[test_case(Value::Int(10), Value::Int(5), Value::Bool(false) ; "lt")]
    #[test_case(Value::Int(5), Value::Int(10), Value::Bool(true) ; "lt_true")]
    fn test_value_lt(left: Value, right: Value, expected: Value) {
        assert_eq!(left.lt(&right), Some(expected));
    }

    #[test_case(Value::Int(10), Value::Int(5), Value::Bool(true) ; "gt")]
    #[test_case(Value::Int(5), Value::Int(10), Value::Bool(false) ; "gt_false")]
    fn test_value_gt(left: Value, right: Value, expected: Value) {
        assert_eq!(left.gt(&right), Some(expected));
    }

    #[test_case(Value::Int(10), Value::Int(5), Value::Bool(false) ; "eq_false")]
    #[test_case(Value::Int(5), Value::Int(5), Value::Bool(true) ; "eq_true")]
    fn test_value_eq(left: Value, right: Value, expected: Value) {
        assert_eq!(left.eq(&right), expected);
    }

    #[test_case(Value::Int(2), Value::Int(3), Value::Float(8.0) ; "int_power")]
    fn test_value_power(base: Value, exp: Value, expected: Value) {
        assert_eq!(base.pow(&exp), Some(expected));
    }

    #[test_case(Value::Null, false ; "as_bool_null")]
    #[test_case(Value::Bool(true), true ; "as_bool_true")]
    #[test_case(Value::Bool(false), false ; "as_bool_false")]
    #[test_case(Value::Int(0), false ; "as_bool_int_zero")]
    #[test_case(Value::Int(5), true ; "as_bool_int_nonzero")]
    #[test_case(Value::Float(0.0), false ; "as_bool_float_zero")]
    #[test_case(Value::Float(1.5), true ; "as_bool_float_nonzero")]
    #[test_case(Value::String("".to_string()), false ; "as_bool_empty_string")]
    #[test_case(Value::String("hello".to_string()), true ; "as_bool_non_empty_string")]
    fn test_as_bool(input: Value, expected: bool) {
        assert_eq!(input.as_bool(), expected);
    }

    #[test_case(Value::Null, Value::Null, Some(std::cmp::Ordering::Equal) ; "compare_null_eq")]
    #[test_case(Value::Null, Value::Int(1), Some(std::cmp::Ordering::Less) ; "compare_null_less")]
    #[test_case(Value::Int(1), Value::Null, Some(std::cmp::Ordering::Greater) ; "compare_null_greater")]
    #[test_case(Value::Bool(false), Value::Bool(true), Some(std::cmp::Ordering::Less) ; "compare_bool_less")]
    #[test_case(Value::Bool(true), Value::Bool(true), Some(std::cmp::Ordering::Equal) ; "compare_bool_eq")]
    #[test_case(Value::Bool(true), Value::Bool(false), Some(std::cmp::Ordering::Greater) ; "compare_bool_greater")]
    #[test_case(Value::Int(1), Value::Int(2), Some(std::cmp::Ordering::Less) ; "compare_int_less")]
    #[test_case(Value::Int(5), Value::Int(5), Some(std::cmp::Ordering::Equal) ; "compare_int_eq")]
    #[test_case(Value::Int(10), Value::Int(3), Some(std::cmp::Ordering::Greater) ; "compare_int_greater")]
    #[test_case(Value::Float(1.5), Value::Float(2.5), Some(std::cmp::Ordering::Less) ; "compare_float_less")]
    #[test_case(Value::Float(3.0), Value::Float(3.0), Some(std::cmp::Ordering::Equal) ; "compare_float_eq")]
    fn test_value_compare(
        left: Value,
        right: Value,
        expected: Option<std::cmp::Ordering>,
    ) {
        assert_eq!(left.compare(&right), expected);
    }

    #[test_case(Value::Int(1), Value::Float(2.0), Some(std::cmp::Ordering::Less) ; "int_float_less")]
    #[test_case(Value::Int(5), Value::Float(5.0), Some(std::cmp::Ordering::Equal) ; "int_float_eq")]
    #[test_case(Value::Float(1.5), Value::Int(2), Some(std::cmp::Ordering::Less) ; "float_int_less")]
    fn test_value_compare_int_float(
        left: Value,
        right: Value,
        expected: Option<std::cmp::Ordering>,
    ) {
        assert_eq!(left.compare(&right), expected);
    }

    #[test_case("apple", "banana", Some(std::cmp::Ordering::Less) ; "string_less")]
    #[test_case("zebra", "apple", Some(std::cmp::Ordering::Greater) ; "string_greater")]
    #[test_case("same", "same", Some(std::cmp::Ordering::Equal) ; "string_eq")]
    fn test_value_compare_string(
        left: &str,
        right: &str,
        expected: Option<std::cmp::Ordering>,
    ) {
        assert_eq!(
            Value::String(left.to_string()).compare(&Value::String(right.to_string())),
            expected
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

    // Type priority: null < bool < int/float < string < list
    #[test_case(Value::Null, Value::Bool(true), Some(std::cmp::Ordering::Less) ; "null_lt_bool")]
    #[test_case(Value::Bool(false), Value::Int(0), Some(std::cmp::Ordering::Less) ; "bool_lt_int")]
    #[test_case(Value::Int(100), Value::String("a".to_string()), Some(std::cmp::Ordering::Less) ; "int_lt_string")]
    #[test_case(Value::String("z".to_string()), Value::List(vec![]), Some(std::cmp::Ordering::Less) ; "string_lt_list")]
    fn test_value_compare_mixed_types(
        left: Value,
        right: Value,
        expected: Option<std::cmp::Ordering>,
    ) {
        assert_eq!(left.compare(&right), expected);
    }

    // Test lexicographical ordering with composite keys like [r.nth(0), r.nth(1)]
    #[test_case(
        vec![Value::Int(1), Value::String("a".to_string())],
        vec![Value::Int(1), Value::String("c".to_string())],
        Some(std::cmp::Ordering::Less) ; "lex_same_first_diff_second"
    )]
    #[test_case(
        vec![Value::Int(1), Value::String("a".to_string())],
        vec![Value::Int(2), Value::String("b".to_string())],
        Some(std::cmp::Ordering::Less) ; "lex_diff_first"
    )]
    #[test_case(
        vec![Value::Int(1), Value::String("c".to_string())],
        vec![Value::Int(2), Value::String("b".to_string())],
        Some(std::cmp::Ordering::Less) ; "lex_same_first_greater_second"
    )]
    fn test_value_compare_list_lexicographical(
        left: Vec<Value>,
        right: Vec<Value>,
        expected: Option<std::cmp::Ordering>,
    ) {
        assert_eq!(Value::List(left).compare(&Value::List(right)), expected);
    }

    #[test_case(Value::Int(42), true ; "is_numeric_int")]
    #[test_case(Value::Float(3.14), true ; "is_numeric_float")]
    #[test_case(Value::Null, false ; "is_numeric_null")]
    #[test_case(Value::Bool(true), false ; "is_numeric_bool")]
    #[test_case(Value::String("123".to_string()), false ; "is_numeric_string")]
    #[test_case(Value::List(vec![]), false ; "is_numeric_list")]
    fn test_is_numeric(input: Value, expected: bool) {
        assert_eq!(input.is_numeric(), expected);
    }

    #[test_case(Value::Null, true ; "is_null_null")]
    #[test_case(Value::Int(42), false ; "is_null_int")]
    #[test_case(Value::Bool(false), false ; "is_null_bool")]
    #[test_case(Value::String("".to_string()), false ; "is_null_string")]
    fn test_is_null(input: Value, expected: bool) {
        assert_eq!(input.is_null(), expected);
    }

    #[test_case(Value::Null, "null" ; "type_name_null")]
    #[test_case(Value::Bool(true), "bool" ; "type_name_bool")]
    #[test_case(Value::Int(42), "int" ; "type_name_int")]
    #[test_case(Value::Float(3.14), "float" ; "type_name_float")]
    #[test_case(Value::String("hello".to_string()), "string" ; "type_name_string")]
    #[test_case(Value::List(vec![]), "list" ; "type_name_list")]
    fn test_type_name(input: Value, expected: &str) {
        assert_eq!(input.type_name(), expected);
    }

    #[test_case(Value::Null, "null" ; "to_string_null")]
    #[test_case(Value::Bool(true), "true" ; "to_string_true")]
    #[test_case(Value::Bool(false), "false" ; "to_string_false")]
    #[test_case(Value::Int(42), "42" ; "to_string_int")]
    #[test_case(Value::Float(3.14), "3.14" ; "to_string_float")]
    #[test_case(Value::String("hello".to_string()), "hello" ; "to_string_string")]
    fn test_to_string(input: Value, expected: &str) {
        assert_eq!(input.to_string(), expected);
    }

    #[test_case(Value::String("hello".to_string()), "hello" ; "as_string_string")]
    #[test_case(Value::Int(42), "42" ; "as_string_int")]
    #[test_case(Value::Null, "null" ; "as_string_null")]
    #[test_case(Value::Bool(true), "true" ; "as_string_bool")]
    fn test_as_string(input: Value, expected: &str) {
        assert_eq!(input.as_string(), expected);
    }

    #[test_case(Value::Int(42), Some(42) ; "as_int_int")]
    #[test_case(Value::Float(3.7), Some(3) ; "as_int_float")]
    #[test_case(Value::String("123".to_string()), Some(123) ; "as_int_string_parses")]
    #[test_case(Value::String("abc".to_string()), None ; "as_int_string_no_parse")]
    #[test_case(Value::Bool(true), Some(1) ; "as_int_bool_true")]
    #[test_case(Value::Bool(false), Some(0) ; "as_int_bool_false")]
    #[test_case(Value::Null, None ; "as_int_null")]
    #[test_case(Value::List(vec![]), None ; "as_int_list")]
    fn test_as_int(input: Value, expected: Option<i64>) {
        assert_eq!(input.as_int(), expected);
    }

    #[test_case(Value::Float(3.14), Some(3.14) ; "as_float_float")]
    #[test_case(Value::Int(42), None ; "as_float_int")]
    #[test_case(Value::Null, None ; "as_float_null")]
    fn test_as_float(input: Value, expected: Option<f64>) {
        assert_eq!(input.as_float(), expected);
    }

    #[test_case(Value::List(vec![]), false ; "as_bool_empty_list")]
    #[test_case(Value::List(vec![Value::Int(1)]), true ; "as_bool_non_empty_list")]
    fn test_as_bool_with_list(input: Value, expected: bool) {
        assert_eq!(input.as_bool(), expected);
    }

    #[test]
    fn test_as_bool_with_datetime() {
        use chrono::Utc;
        assert_eq!(Value::DateTime(Utc::now()).as_bool(), true);
    }

    #[test]
    fn test_as_bool_with_lambda() {
        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: crate::libs::expr::parser::ast::Expr::LambdaParam("x".to_string()),
            captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
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
            captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
        });
        let lambda2 = Value::Lambda(LambdaValue {
            params: vec!["y".to_string()],
            body: crate::libs::expr::parser::ast::Expr::LambdaParam("y".to_string()),
            captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
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
            captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
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
            captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
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

    #[test]
    fn test_value_concat() {
        // Test string concatenation
        let a = Value::String("hello".to_string());
        let b = Value::String("world".to_string());
        assert_eq!(
            a.concat(&b).unwrap(),
            Value::String("helloworld".to_string())
        );

        // Test concatenation with non-strings (should convert to string)
        let a = Value::Int(42);
        let b = Value::String("test".to_string());
        assert_eq!(a.concat(&b).unwrap(), Value::String("42test".to_string()));
    }

    #[test]
    fn test_value_modulo() {
        // Test integer modulo
        let a = Value::Int(10);
        let b = Value::Int(3);
        assert_eq!(a.modulo(&b).unwrap(), Value::Int(1));

        // Test float modulo
        let a = Value::Float(10.5);
        let b = Value::Float(3.0);
        assert_eq!(a.modulo(&b).unwrap(), Value::Float(1.5));

        // Test mixed types
        let a = Value::Int(10);
        let b = Value::Float(3.0);
        assert_eq!(a.modulo(&b).unwrap(), Value::Float(1.0));
    }

    #[test]
    fn test_value_modulo_by_zero() {
        let a = Value::Int(10);
        let b = Value::Int(0);
        assert!(matches!(a.modulo(&b), Err(EvalError::DivisionByZero)));

        let a = Value::Float(10.0);
        let b = Value::Float(0.0);
        assert!(matches!(a.modulo(&b), Err(EvalError::DivisionByZero)));
    }

    #[test]
    fn test_value_modulo_type_error() {
        let a = Value::String("hello".to_string());
        let b = Value::Int(3);
        assert!(matches!(a.modulo(&b), Err(EvalError::TypeError(_))));
    }

    #[test]
    fn test_as_f64_various_types() {
        assert_eq!(Value::Int(42).as_f64(), Some(42.0));
        assert_eq!(Value::Float(3.14).as_f64(), Some(3.14));
        assert_eq!(Value::Null.as_f64(), None);
        assert_eq!(Value::Bool(true).as_f64(), None);
        assert_eq!(Value::String("hello".to_string()).as_f64(), None);
        assert_eq!(Value::List(vec![]).as_f64(), None);
    }

    #[test]
    fn test_as_int_various_types() {
        assert_eq!(Value::Int(42).as_int(), Some(42));
        assert_eq!(Value::Float(3.7).as_int(), Some(3)); // Truncates
        assert_eq!(Value::String("123".to_string()).as_int(), Some(123));
        assert_eq!(Value::String("abc".to_string()).as_int(), None);
        assert_eq!(Value::Bool(true).as_int(), Some(1));
        assert_eq!(Value::Bool(false).as_int(), Some(0));
        assert_eq!(Value::Null.as_int(), None);
    }

    #[test]
    fn test_as_string_various_types() {
        assert_eq!(Value::String("hello".to_string()).as_string(), "hello");
        assert_eq!(Value::Int(42).as_string(), "42");
        assert_eq!(Value::Float(3.14).as_string(), "3.14");
        assert_eq!(Value::Bool(true).as_string(), "true");
        assert_eq!(Value::Null.as_string(), "null");
    }

    #[test]
    fn test_value_equality() {
        // Same types
        assert_eq!(Value::Int(42), Value::Int(42));
        assert_ne!(Value::Int(42), Value::Int(43));
        assert_eq!(Value::Float(3.14), Value::Float(3.14));
        assert_eq!(
            Value::String("hello".to_string()),
            Value::String("hello".to_string())
        );
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(Value::Null, Value::Null);

        // Different types
        assert_ne!(Value::Int(42), Value::Float(42.0));
        assert_ne!(Value::Int(42), Value::String("42".to_string()));
        assert_ne!(Value::Bool(true), Value::Int(1));
    }

    #[test]
    fn test_list_equality() {
        let a = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let b = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let c = Value::List(vec![Value::Int(1), Value::Int(3)]);
        let d = Value::List(vec![Value::Int(1)]);

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn test_nested_list_equality() {
        let a = Value::List(vec![
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::Int(3),
        ]);
        let b = Value::List(vec![
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::Int(3),
        ]);
        let c = Value::List(vec![
            Value::List(vec![Value::Int(1), Value::Int(3)]),
            Value::Int(3),
        ]);

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_datetime_equality() {
        use chrono::Utc;
        let dt1 = Utc::now();
        let dt2 = dt1;
        let dt3 = Utc::now();

        assert_eq!(Value::DateTime(dt1), Value::DateTime(dt2));
        // Note: dt3 might be equal or not depending on timing
        // Just verify the comparison works
        let _ = Value::DateTime(dt1) == Value::DateTime(dt3);
    }

    #[test]
    fn test_lambda_equality() {
        use crate::libs::expr::parser::ast::Expr;
        let lambda1 = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::Int(1),
            captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
        });
        let lambda2 = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: Expr::Int(1),
            captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
        });
        let lambda3 = Value::Lambda(LambdaValue {
            params: vec!["y".to_string()],
            body: Expr::Int(2),
            captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
        });

        assert_eq!(lambda1, lambda2);
        assert_ne!(lambda1, lambda3);
    }

    #[test]
    fn test_arithmetic_with_negative_numbers() {
        let a = Value::Int(-10);
        let b = Value::Int(-3);

        assert_eq!((a.clone() + b.clone()).unwrap(), Value::Int(-13));
        assert_eq!((a.clone() - b.clone()).unwrap(), Value::Int(-7));
        assert_eq!((a.clone() * b.clone()).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_arithmetic_with_zero() {
        let a = Value::Int(10);
        let zero = Value::Int(0);

        assert_eq!((a.clone() + zero.clone()).unwrap(), Value::Int(10));
        assert_eq!((a.clone() - zero.clone()).unwrap(), Value::Int(10));
        assert_eq!((zero.clone() * a.clone()).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_float_special_values() {
        // Test with infinity
        let inf = Value::Float(f64::INFINITY);
        let neg_inf = Value::Float(f64::NEG_INFINITY);
        let num = Value::Float(1.0);

        assert!(
            matches!(inf.clone() + num.clone(), Some(Value::Float(f)) if f.is_infinite())
        );
        assert!(
            matches!(neg_inf.clone() + num.clone(), Some(Value::Float(f)) if f.is_infinite())
        );

        // Test with NaN
        let nan = Value::Float(f64::NAN);
        assert!(
            matches!(nan.clone() + num.clone(), Some(Value::Float(f)) if f.is_nan())
        );

        // NaN comparisons
        assert_eq!(nan.eq(&nan), Value::Bool(false)); // NaN != NaN
    }

    #[test]
    fn test_compare_empty_lists() {
        let empty1: Vec<Value> = vec![];
        let empty2: Vec<Value> = vec![];
        let non_empty = Value::List(vec![Value::Int(1)]);

        assert_eq!(
            Value::List(empty1.clone()).compare(&Value::List(empty2)),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            Value::List(empty1).compare(&non_empty),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_compare_lists_different_lengths() {
        let a = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let b = Value::List(vec![Value::Int(1)]);

        assert_eq!(a.compare(&b), Some(std::cmp::Ordering::Greater));
        assert_eq!(b.compare(&a), Some(std::cmp::Ordering::Less));
    }

    #[test]
    fn test_compare_list_with_incomparable_nested() {
        // Lists with lambda elements should not be comparable
        let lambda = Value::Lambda(LambdaValue {
            params: vec![],
            body: crate::libs::expr::parser::ast::Expr::Int(1),
            captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
        });
        let list1 = Value::List(vec![lambda.clone()]);
        let list2 = Value::List(vec![lambda.clone()]);

        assert_eq!(list1.compare(&list2), None);
    }

    #[test]
    fn test_value_clone() {
        let original = Value::List(vec![
            Value::Int(1),
            Value::String("hello".to_string()),
            Value::Bool(true),
        ]);
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_lambda_clone() {
        let lambda = LambdaValue {
            params: vec!["x".to_string(), "y".to_string()],
            body: crate::libs::expr::parser::ast::Expr::Binary {
                op: crate::libs::expr::parser::ast::BinaryOp::Add,
                left: Box::new(crate::libs::expr::parser::ast::Expr::LambdaParam(
                    "x".to_string(),
                )),
                right: Box::new(crate::libs::expr::parser::ast::Expr::LambdaParam(
                    "y".to_string(),
                )),
            },
            captured_vars: {
                let mut map = HashMap::with_hasher(ahash::RandomState::new());
                map.insert("outer".to_string(), Value::Int(42));
                map
            },
        };

        let cloned = lambda.clone();
        assert_eq!(lambda.params, cloned.params);
        assert_eq!(lambda.captured_vars, cloned.captured_vars);
    }

    #[test]
    fn test_rem_trait() {
        let a = Value::Int(10);
        let b = Value::Int(3);
        assert_eq!(a % b, Some(Value::Int(1)));

        let a = Value::Float(10.5);
        let b = Value::Float(3.0);
        assert_eq!(a % b, Some(Value::Float(1.5)));

        let a = Value::Int(10);
        let b = Value::Int(0);
        assert_eq!(a % b, None);
    }

    #[test]
    fn test_add_trait() {
        let a = Value::Int(10);
        let b = Value::Int(20);
        assert_eq!(a + b, Some(Value::Int(30)));

        let a = Value::Float(1.5);
        let b = Value::Float(2.5);
        assert_eq!(a + b, Some(Value::Float(4.0)));

        let a = Value::String("hello".to_string());
        let b = Value::Int(1);
        assert_eq!(a + b, None);
    }

    #[test]
    fn test_sub_trait() {
        let a = Value::Int(20);
        let b = Value::Int(10);
        assert_eq!(a - b, Some(Value::Int(10)));

        let a = Value::Float(5.5);
        let b = Value::Float(2.5);
        assert_eq!(a - b, Some(Value::Float(3.0)));
    }

    #[test]
    fn test_mul_trait() {
        let a = Value::Int(5);
        let b = Value::Int(6);
        assert_eq!(a * b, Some(Value::Int(30)));

        let a = Value::Float(2.5);
        let b = Value::Float(4.0);
        assert_eq!(a * b, Some(Value::Float(10.0)));
    }

    #[test]
    fn test_div_trait() {
        let a = Value::Float(10.0);
        let b = Value::Float(2.0);
        assert_eq!(a / b, Some(Value::Float(5.0)));

        let a = Value::Float(10.0);
        let b = Value::Float(0.0);
        assert_eq!(a / b, None);
    }

    #[test]
    fn test_power_with_zero() {
        let a = Value::Int(5);
        let b = Value::Int(0);
        assert_eq!(a.pow(&b), Some(Value::Float(1.0)));

        let a = Value::Int(0);
        let b = Value::Int(5);
        assert_eq!(a.pow(&b), Some(Value::Float(0.0)));
    }

    #[test]
    fn test_power_negative() {
        let a = Value::Int(2);
        let b = Value::Int(-1);
        assert_eq!(a.pow(&b), Some(Value::Float(0.5)));
    }

    #[test]
    fn test_compare_bool_with_int() {
        // Different types should use type priority
        let bool_val = Value::Bool(true);
        let int_val = Value::Int(1);

        assert_eq!(bool_val.compare(&int_val), Some(std::cmp::Ordering::Less));
        assert_eq!(
            int_val.compare(&bool_val),
            Some(std::cmp::Ordering::Greater)
        );
    }

    #[test]
    fn test_compare_string_with_list() {
        let string_val = Value::String("hello".to_string());
        let list_val = Value::List(vec![]);

        assert_eq!(
            string_val.compare(&list_val),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            list_val.compare(&string_val),
            Some(std::cmp::Ordering::Greater)
        );
    }

    #[test]
    fn test_is_null_various_types() {
        assert!(Value::Null.is_null());
        assert!(!Value::Int(0).is_null());
        assert!(!Value::Bool(false).is_null());
        assert!(!Value::String("".to_string()).is_null());
        assert!(!Value::List(vec![]).is_null());
    }

    #[test]
    fn test_as_bool_edge_cases() {
        // Zero is falsy
        assert!(!Value::Int(0).as_bool());
        assert!(!Value::Float(0.0).as_bool());

        // Non-zero is truthy
        assert!(Value::Int(-1).as_bool());
        assert!(Value::Float(-0.1).as_bool());
        assert!(Value::Float(f64::MIN_POSITIVE).as_bool());

        // Empty/null is falsy
        assert!(!Value::Null.as_bool());
        assert!(!Value::String("".to_string()).as_bool());
        assert!(!Value::List(vec![]).as_bool());

        // Non-empty is truthy
        assert!(Value::String(" ".to_string()).as_bool());
        assert!(Value::List(vec![Value::Null]).as_bool());
    }

    #[test]
    fn test_list_to_string() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let str = list.to_string();
        assert!(str.contains("Int(1)"));
        assert!(str.contains("Int(2)"));
    }

    #[test]
    fn test_datetime_to_string() {
        use chrono::Utc;
        let dt = Utc::now();
        let val = Value::DateTime(dt);
        assert_eq!(val.to_string(), dt.to_rfc3339());
    }

    #[test]
    fn test_lambda_to_string() {
        let lambda = Value::Lambda(LambdaValue {
            params: vec!["x".to_string()],
            body: crate::libs::expr::parser::ast::Expr::Int(1),
            captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
        });
        assert_eq!(lambda.to_string(), "<lambda>");
    }

    #[test]
    fn test_null_to_string() {
        assert_eq!(Value::Null.to_string(), "null");
    }

    #[test]
    fn test_bool_to_string() {
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_int_to_string() {
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Int(-100).to_string(), "-100");
        assert_eq!(Value::Int(0).to_string(), "0");
    }

    #[test]
    fn test_float_to_string() {
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
        assert_eq!(Value::Float(-2.5).to_string(), "-2.5");
    }

    #[test]
    fn test_string_to_string() {
        assert_eq!(Value::String("hello".to_string()).to_string(), "hello");
        assert_eq!(Value::String("".to_string()).to_string(), "");
    }

    #[test]
    fn test_display_trait() {
        use std::fmt::Write;

        let mut buf = String::new();
        write!(&mut buf, "{}", Value::Int(42)).unwrap();
        assert_eq!(buf, "42");

        buf.clear();
        write!(&mut buf, "{}", Value::String("test".to_string())).unwrap();
        assert_eq!(buf, "test");

        buf.clear();
        write!(&mut buf, "{}", Value::Null).unwrap();
        assert_eq!(buf, "null");
    }

    #[test]
    fn test_le_comparison() {
        let a = Value::Int(5);
        let b = Value::Int(5);
        let c = Value::Int(10);

        assert_eq!(a.le(&b), Some(Value::Bool(true)));
        assert_eq!(a.le(&c), Some(Value::Bool(true)));
        assert_eq!(c.le(&a), Some(Value::Bool(false)));
    }

    #[test]
    fn test_ge_comparison() {
        let a = Value::Int(10);
        let b = Value::Int(10);
        let c = Value::Int(5);

        assert_eq!(a.ge(&b), Some(Value::Bool(true)));
        assert_eq!(a.ge(&c), Some(Value::Bool(true)));
        assert_eq!(c.ge(&a), Some(Value::Bool(false)));
    }

    #[test]
    fn test_compare_same_type_different_values() {
        // Int
        assert_eq!(
            Value::Int(5).compare(&Value::Int(10)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Int(10).compare(&Value::Int(5)),
            Some(std::cmp::Ordering::Greater)
        );

        // Float
        assert_eq!(
            Value::Float(1.5).compare(&Value::Float(2.5)),
            Some(std::cmp::Ordering::Less)
        );

        // String
        assert_eq!(
            Value::String("apple".to_string())
                .compare(&Value::String("banana".to_string())),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_compare_null_with_others() {
        assert_eq!(
            Value::Null.compare(&Value::Null),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            Value::Null.compare(&Value::Bool(false)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Null.compare(&Value::Int(0)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Null.compare(&Value::Float(0.0)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Null.compare(&Value::String("".to_string())),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Null.compare(&Value::List(vec![])),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_type_name_all_types() {
        use chrono::Utc;

        assert_eq!(Value::Null.type_name(), "null");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::Int(42).type_name(), "int");
        assert_eq!(Value::Float(3.14).type_name(), "float");
        assert_eq!(Value::String("hello".to_string()).type_name(), "string");
        assert_eq!(Value::List(vec![]).type_name(), "list");
        assert_eq!(Value::DateTime(Utc::now()).type_name(), "datetime");
        assert_eq!(
            Value::Lambda(LambdaValue {
                params: vec![],
                body: crate::libs::expr::parser::ast::Expr::Int(1),
                captured_vars: HashMap::with_hasher(ahash::RandomState::new()),
            })
            .type_name(),
            "lambda"
        );
    }
}
