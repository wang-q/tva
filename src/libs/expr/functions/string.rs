use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;

pub fn trim(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        Value::Null => Ok(Value::Null),
        v => Ok(Value::String(v.to_string().trim().to_string())),
    }
}

pub fn upper(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        Value::Null => Ok(Value::Null),
        v => Ok(Value::String(v.to_string().to_uppercase())),
    }
}

pub fn lower(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        Value::Null => Ok(Value::Null),
        v => Ok(Value::String(v.to_string().to_lowercase())),
    }
}

pub fn len(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::List(list) => Ok(Value::Int(list.len() as i64)),
        Value::Null => Ok(Value::Int(0)),
        v => Ok(Value::Int(v.to_string().len() as i64)),
    }
}

pub fn substr(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let start = args[1].as_int().unwrap_or(0) as usize;
    let len = args[2].as_int().unwrap_or(s.len() as i64) as usize;

    if start >= s.len() {
        return Ok(Value::String(String::new()));
    }

    let end = (start + len).min(s.len());
    Ok(Value::String(s[start..end].to_string()))
}

pub fn split(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let delim = args[1].as_string();

    let parts: Vec<Value> = s
        .split(&delim)
        .map(|p| Value::String(p.to_string()))
        .collect();
    Ok(Value::List(parts))
}

pub fn contains(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let substr = args[1].as_string();
    Ok(Value::Bool(s.contains(&substr)))
}

pub fn starts_with(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let prefix = args[1].as_string();
    Ok(Value::Bool(s.starts_with(&prefix)))
}

pub fn ends_with(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let suffix = args[1].as_string();
    Ok(Value::Bool(s.ends_with(&suffix)))
}

pub fn replace(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let from = args[1].as_string();
    let to = args[2].as_string();
    Ok(Value::String(s.replace(&from, &to)))
}

pub fn wordcount(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let count = s.split_whitespace().count() as i64;
    Ok(Value::Int(count))
}

pub fn char_len(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    Ok(Value::Int(s.chars().count() as i64))
}

pub fn truncate(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let len = match &args[1] {
        Value::Int(n) => *n as usize,
        Value::Float(f) => f.round() as usize,
        v => {
            return Err(EvalError::TypeError(format!(
                "truncate: length must be a number, got {}",
                v.type_name()
            )))
        }
    };
    let end = if args.len() > 2 {
        args[2].as_string()
    } else {
        "...".to_string()
    };

    if s.len() <= len {
        Ok(Value::String(s))
    } else {
        let truncated = &s[..len.saturating_sub(end.len())];
        Ok(Value::String(format!("{}{}", truncated, end)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim() {
        assert_eq!(
            trim(&[Value::String("  hello  ".to_string())]).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_upper() {
        assert_eq!(
            upper(&[Value::String("hello".to_string())]).unwrap(),
            Value::String("HELLO".to_string())
        );
    }

    #[test]
    fn test_lower() {
        assert_eq!(
            lower(&[Value::String("HELLO".to_string())]).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_len() {
        assert_eq!(
            len(&[Value::String("hello".to_string())]).unwrap(),
            Value::Int(5)
        );
        // Null returns 0
        assert_eq!(len(&[Value::Null]).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_substr() {
        assert_eq!(
            substr(&[
                Value::String("hello world".to_string()),
                Value::Int(0),
                Value::Int(5),
            ])
            .unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_split() {
        match split(&[
            Value::String("a,b,c".to_string()),
            Value::String(",".to_string()),
        ])
        .unwrap()
        {
            Value::List(vals) => {
                assert_eq!(vals.len(), 3);
                assert_eq!(vals[0], Value::String("a".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_contains() {
        assert_eq!(
            contains(&[
                Value::String("hello world".to_string()),
                Value::String("world".to_string()),
            ])
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_starts_with() {
        assert_eq!(
            starts_with(&[
                Value::String("hello".to_string()),
                Value::String("he".to_string()),
            ])
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_ends_with() {
        assert_eq!(
            ends_with(&[
                Value::String("hello".to_string()),
                Value::String("lo".to_string()),
            ])
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_replace() {
        assert_eq!(
            replace(&[
                Value::String("hello world".to_string()),
                Value::String("world".to_string()),
                Value::String("rust".to_string()),
            ])
            .unwrap(),
            Value::String("hello rust".to_string())
        );
    }

    #[test]
    fn test_truncate() {
        // "hello world" is 11 bytes, limit 8, end "..." is 3 bytes
        assert_eq!(
            truncate(&[Value::String("hello world".to_string()), Value::Int(8)])
                .unwrap(),
            Value::String("hello...".to_string())
        );

        // Custom ending
        assert_eq!(
            truncate(&[
                Value::String("hello world".to_string()),
                Value::Int(8),
                Value::String(">>".to_string()),
            ])
            .unwrap(),
            Value::String("hello >>".to_string())
        );

        // String shorter than limit - return as-is
        assert_eq!(
            truncate(&[Value::String("hi".to_string()), Value::Int(10)]).unwrap(),
            Value::String("hi".to_string())
        );
    }

    #[test]
    fn test_wordcount() {
        assert_eq!(
            wordcount(&[Value::String("hello world foo bar".to_string())]).unwrap(),
            Value::Int(4)
        );
        assert_eq!(
            wordcount(&[Value::String("   multiple   spaces   ".to_string())]).unwrap(),
            Value::Int(2)
        );
        assert_eq!(
            wordcount(&[Value::String("".to_string())]).unwrap(),
            Value::Int(0)
        );
    }

    #[test]
    fn test_char_len() {
        assert_eq!(
            char_len(&[Value::String("hello".to_string())]).unwrap(),
            Value::Int(5)
        );
        // UTF-8 characters
        assert_eq!(
            char_len(&[Value::String("你好世界".to_string())]).unwrap(),
            Value::Int(4)
        );
    }

    #[test]
    fn test_trim_null() {
        assert_eq!(trim(&[Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn test_len_null() {
        assert_eq!(len(&[Value::Null]).unwrap(), Value::Int(0));
    }
}
