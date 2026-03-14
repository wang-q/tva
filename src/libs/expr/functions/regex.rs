use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use regex::Regex;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_match() {
        let result = regex_match(&[
            Value::String("hello world".to_string()),
            Value::String(r"hello.*".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));

        let result = regex_match(&[
            Value::String("hello world".to_string()),
            Value::String(r"^foo".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_regex_extract() {
        // Extract full match (group 0)
        let result = regex_extract(&[
            Value::String("hello 123 world".to_string()),
            Value::String(r"\d+".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::String("123".to_string()));

        // Extract specific group
        let result = regex_extract(&[
            Value::String("hello 123 world".to_string()),
            Value::String(r"(\d+)".to_string()),
            Value::Int(1),
        ]);
        assert_eq!(result.unwrap(), Value::String("123".to_string()));

        // No match
        let result = regex_extract(&[
            Value::String("hello world".to_string()),
            Value::String(r"\d+".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_regex_replace() {
        let result = regex_replace(&[
            Value::String("hello 123 world 456".to_string()),
            Value::String(r"\d+".to_string()),
            Value::String("XXX".to_string()),
        ]);
        assert_eq!(
            result.unwrap(),
            Value::String("hello XXX world XXX".to_string())
        );

        // Replace with capture group reference
        let result = regex_replace(&[
            Value::String("hello world".to_string()),
            Value::String(r"(world)".to_string()),
            Value::String("$1!".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::String("hello world!".to_string()));
    }
}

pub fn regex_match(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let pattern = args[1].as_string();

    match Regex::new(&pattern) {
        Ok(re) => Ok(Value::Bool(re.is_match(&s))),
        Err(e) => Err(EvalError::TypeError(format!(
            "regex_match: invalid pattern: {}",
            e
        ))),
    }
}

pub fn regex_extract(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let pattern = args[1].as_string();
    let group = if args.len() > 2 {
        match &args[2] {
            Value::Int(n) => *n as usize,
            Value::Float(f) => f.round() as usize,
            v => {
                return Err(EvalError::TypeError(format!(
                    "regex_extract: group must be a number, got {}",
                    v.type_name()
                )))
            }
        }
    } else {
        0
    };

    match Regex::new(&pattern) {
        Ok(re) => {
            if let Some(caps) = re.captures(&s) {
                if let Some(m) = caps.get(group) {
                    Ok(Value::String(m.as_str().to_string()))
                } else {
                    Ok(Value::Null)
                }
            } else {
                Ok(Value::Null)
            }
        }
        Err(e) => Err(EvalError::TypeError(format!(
            "regex_extract: invalid pattern: {}",
            e
        ))),
    }
}

pub fn regex_replace(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let pattern = args[1].as_string();
    let replacement = args[2].as_string();

    match Regex::new(&pattern) {
        Ok(re) => Ok(Value::String(re.replace_all(&s, &replacement).to_string())),
        Err(e) => Err(EvalError::TypeError(format!(
            "regex_replace: invalid pattern: {}",
            e
        ))),
    }
}
