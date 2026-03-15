use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use regex::Regex;

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
    fn test_regex_match_empty_string() {
        // Empty string should match empty pattern
        let result = regex_match(&[
            Value::String("".to_string()),
            Value::String(r"^$".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));

        // Empty string should not match non-empty pattern
        let result = regex_match(&[
            Value::String("".to_string()),
            Value::String(r".+".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_regex_match_case_sensitive() {
        // Regex is case-sensitive by default
        let result = regex_match(&[
            Value::String("Hello World".to_string()),
            Value::String(r"hello".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(false));

        // Case-insensitive match
        let result = regex_match(&[
            Value::String("Hello World".to_string()),
            Value::String(r"(?i)hello".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_regex_match_special_chars() {
        // Special regex characters should be handled
        let result = regex_match(&[
            Value::String("hello.world".to_string()),
            Value::String(r"hello\.world".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));

        // Dot matches any character
        let result = regex_match(&[
            Value::String("helloXworld".to_string()),
            Value::String(r"hello.world".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_regex_match_invalid_pattern() {
        let result = regex_match(&[
            Value::String("hello".to_string()),
            Value::String(r"[invalid".to_string()),
        ]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid pattern"));
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
    fn test_regex_extract_multiple_groups() {
        // Extract from multiple capture groups
        let result = regex_extract(&[
            Value::String("Date: 2024-03-15".to_string()),
            Value::String(r"(\d{4})-(\d{2})-(\d{2})".to_string()),
            Value::Int(0),
        ]);
        assert_eq!(result.unwrap(), Value::String("2024-03-15".to_string()));

        let result = regex_extract(&[
            Value::String("Date: 2024-03-15".to_string()),
            Value::String(r"(\d{4})-(\d{2})-(\d{2})".to_string()),
            Value::Int(1),
        ]);
        assert_eq!(result.unwrap(), Value::String("2024".to_string()));

        let result = regex_extract(&[
            Value::String("Date: 2024-03-15".to_string()),
            Value::String(r"(\d{4})-(\d{2})-(\d{2})".to_string()),
            Value::Int(2),
        ]);
        assert_eq!(result.unwrap(), Value::String("03".to_string()));

        let result = regex_extract(&[
            Value::String("Date: 2024-03-15".to_string()),
            Value::String(r"(\d{4})-(\d{2})-(\d{2})".to_string()),
            Value::Int(3),
        ]);
        assert_eq!(result.unwrap(), Value::String("15".to_string()));
    }

    #[test]
    fn test_regex_extract_group_out_of_bounds() {
        // Group index out of bounds should return null
        let result = regex_extract(&[
            Value::String("hello 123".to_string()),
            Value::String(r"(\d+)".to_string()),
            Value::Int(5),
        ]);
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_regex_extract_with_float_group() {
        // Float group index should be rounded
        let result = regex_extract(&[
            Value::String("hello 123 world".to_string()),
            Value::String(r"(\d+)".to_string()),
            Value::Float(1.0),
        ]);
        assert_eq!(result.unwrap(), Value::String("123".to_string()));
    }

    #[test]
    fn test_regex_extract_invalid_group_type() {
        let result = regex_extract(&[
            Value::String("hello 123".to_string()),
            Value::String(r"(\d+)".to_string()),
            Value::String("group".to_string()),
        ]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("group must be a number"));
    }

    #[test]
    fn test_regex_extract_invalid_pattern() {
        let result = regex_extract(&[
            Value::String("hello".to_string()),
            Value::String(r"[invalid".to_string()),
        ]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid pattern"));
    }

    #[test]
    fn test_regex_extract_empty_string() {
        // Extract from empty string should return null
        let result = regex_extract(&[
            Value::String("".to_string()),
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

    #[test]
    fn test_regex_replace_no_match() {
        // No match should return original string
        let result = regex_replace(&[
            Value::String("hello world".to_string()),
            Value::String(r"\d+".to_string()),
            Value::String("XXX".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::String("hello world".to_string()));
    }

    #[test]
    fn test_regex_replace_empty_pattern() {
        // Empty pattern should match at every position
        let result = regex_replace(&[
            Value::String("ab".to_string()),
            Value::String(r"".to_string()),
            Value::String("-".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::String("-a-b-".to_string()));
    }

    #[test]
    fn test_regex_replace_empty_replacement() {
        // Empty replacement should delete matches
        let result = regex_replace(&[
            Value::String("hello 123 world 456".to_string()),
            Value::String(r"\d+".to_string()),
            Value::String("".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::String("hello  world ".to_string()));
    }

    #[test]
    fn test_regex_replace_special_chars() {
        // Replacement with capture group reference
        let result = regex_replace(&[
            Value::String("hello world".to_string()),
            Value::String(r"(world)".to_string()),
            Value::String("[$1]".to_string()),
        ]);
        // $1 is replaced with the captured group
        assert_eq!(result.unwrap(), Value::String("hello [world]".to_string()));

        // Escape $ in replacement using $$
        let result = regex_replace(&[
            Value::String("hello world".to_string()),
            Value::String(r"world".to_string()),
            Value::String(r"$$100".to_string()),
        ]);
        // $$ becomes literal $
        assert_eq!(result.unwrap(), Value::String("hello $100".to_string()));
    }

    #[test]
    fn test_regex_replace_invalid_pattern() {
        let result = regex_replace(&[
            Value::String("hello".to_string()),
            Value::String(r"[invalid".to_string()),
            Value::String("replacement".to_string()),
        ]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid pattern"));
    }

    #[test]
    fn test_regex_replace_with_named_groups() {
        // Named capture groups
        let result = regex_replace(&[
            Value::String("hello world".to_string()),
            Value::String(r"(?<name>world)".to_string()),
            Value::String("$name".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::String("hello world".to_string()));
    }

    #[test]
    fn test_regex_complex_patterns() {
        // Email pattern
        let result = regex_match(&[
            Value::String("user@example.com".to_string()),
            Value::String(r"^\S+@\S+\.\S+$".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));

        // URL pattern
        let result = regex_match(&[
            Value::String("https://example.com/path".to_string()),
            Value::String(r"^https?://".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));

        // Phone number extraction
        let result = regex_extract(&[
            Value::String("Call me at 123-456-7890".to_string()),
            Value::String(r"(\d{3}-\d{3}-\d{4})".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::String("123-456-7890".to_string()));
    }

    #[test]
    fn test_regex_unicode() {
        // Unicode support
        let result = regex_match(&[
            Value::String("你好世界".to_string()),
            Value::String(r"你好".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));

        // Unicode character class
        let result = regex_match(&[
            Value::String("你好123".to_string()),
            Value::String(r"\p{Han}+".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_regex_multiline() {
        // Multiline matching with anchors
        let text = "line1\nline2\nline3";
        let result = regex_match(&[
            Value::String(text.to_string()),
            Value::String(r"^line2$".to_string()),
        ]);
        // By default, ^ and $ match start/end of string, not lines
        assert_eq!(result.unwrap(), Value::Bool(false));

        // With multiline flag
        let result = regex_match(&[
            Value::String(text.to_string()),
            Value::String(r"(?m)^line2$".to_string()),
        ]);
        assert_eq!(result.unwrap(), Value::Bool(true));
    }
}
