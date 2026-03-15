use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::runtime::EvalError;
use base64::Engine;
use sha2::{Digest, Sha256};

pub fn md5(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let digest = md5::compute(s.as_bytes());
    Ok(Value::String(format!("{:x}", digest)))
}

pub fn sha256(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    Ok(Value::String(format!("{:x}", result)))
}

pub fn base64_encode(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    Ok(Value::String(
        base64::engine::general_purpose::STANDARD.encode(s.as_bytes()),
    ))
}

pub fn base64_decode(args: &[Value]) -> Result<Value, EvalError> {
    let s = args[0].as_string();
    match base64::engine::general_purpose::STANDARD.decode(s.as_bytes()) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(decoded) => Ok(Value::String(decoded)),
            Err(_) => Err(EvalError::TypeError(
                "base64_decode: decoded bytes are not valid UTF-8".to_string(),
            )),
        },
        Err(e) => Err(EvalError::TypeError(format!(
            "base64_decode: invalid base64 string: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5() {
        let result = md5(&[Value::String("hello".to_string())]);
        // MD5 of "hello" is 5d41402abc4b2a76b9719d911017c592
        assert_eq!(
            result.unwrap(),
            Value::String("5d41402abc4b2a76b9719d911017c592".to_string())
        );
    }

    #[test]
    fn test_md5_empty_string() {
        // MD5 of empty string is d41d8cd98f00b204e9800998ecf8427e
        let result = md5(&[Value::String("".to_string())]);
        assert_eq!(
            result.unwrap(),
            Value::String("d41d8cd98f00b204e9800998ecf8427e".to_string())
        );
    }

    #[test]
    fn test_md5_long_string() {
        // Test with a longer string
        let result = md5(&[Value::String(
            "The quick brown fox jumps over the lazy dog".to_string(),
        )]);
        assert_eq!(
            result.unwrap(),
            Value::String("9e107d9d372bb6826bd81d3542a419d6".to_string())
        );
    }

    #[test]
    fn test_md5_unicode() {
        // Test with unicode string
        let result = md5(&[Value::String("你好世界".to_string())]);
        assert!(result.is_ok());
        // Verify it's a valid hex string of correct length (32 chars)
        let hash = result.unwrap().as_string();
        assert_eq!(hash.len(), 32);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256() {
        let result = sha256(&[Value::String("hello".to_string())]);
        // SHA256 of "hello"
        assert_eq!(
            result.unwrap(),
            Value::String(
                "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_sha256_empty_string() {
        // SHA256 of empty string
        let result = sha256(&[Value::String("".to_string())]);
        assert_eq!(
            result.unwrap(),
            Value::String(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_sha256_long_string() {
        // Test with a longer string
        let result = sha256(&[Value::String(
            "The quick brown fox jumps over the lazy dog".to_string(),
        )]);
        assert_eq!(
            result.unwrap(),
            Value::String(
                "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_base64() {
        let result = base64_encode(&[Value::String("hello".to_string())]);
        assert_eq!(result.unwrap(), Value::String("aGVsbG8=".to_string()));

        // Empty string
        let result = base64_encode(&[Value::String("".to_string())]);
        assert_eq!(result.unwrap(), Value::String("".to_string()));
    }

    #[test]
    fn test_base64_special_chars() {
        // Test with special characters
        let result = base64_encode(&[Value::String("Hello, World! @#$%".to_string())]);
        assert_eq!(
            result.unwrap(),
            Value::String("SGVsbG8sIFdvcmxkISBAIyQl".to_string())
        );
    }

    #[test]
    fn test_base64_unicode() {
        // Test with unicode (UTF-8)
        let result = base64_encode(&[Value::String("你好".to_string())]);
        assert_eq!(result.unwrap(), Value::String("5L2g5aW9".to_string()));
    }

    #[test]
    fn test_base64_multiline() {
        // Test with multiline string
        let input = "Line 1\nLine 2\nLine 3".to_string();
        let result = base64_encode(&[Value::String(input.clone())]);
        let encoded = result.unwrap();
        // Verify round-trip
        let decoded = base64_decode(&[encoded]).unwrap();
        assert_eq!(decoded, Value::String(input));
    }

    #[test]
    fn test_unbase64() {
        let result = base64_decode(&[Value::String("aGVsbG8=".to_string())]);
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));

        // Round-trip
        let encoded = base64_encode(&[Value::String("test 123".to_string())]).unwrap();
        let result = base64_decode(&[encoded]);
        assert_eq!(result.unwrap(), Value::String("test 123".to_string()));
    }

    #[test]
    fn test_unbase64_empty() {
        // Decode empty base64
        let result = base64_decode(&[Value::String("".to_string())]);
        assert_eq!(result.unwrap(), Value::String("".to_string()));
    }

    #[test]
    fn test_unbase64_invalid() {
        // Invalid base64 string
        let result = base64_decode(&[Value::String("not-valid-base64!!!".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unbase64_invalid_chars() {
        // Base64 with invalid characters
        let result = base64_decode(&[Value::String("aGVsbG8@".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unbase64_not_utf8() {
        // Valid base64 but decoded bytes are not valid UTF-8
        // This is the base64 encoding of some invalid UTF-8 bytes
        let result = base64_decode(&[Value::String("/w==".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_consistency() {
        // Same input should produce same hash
        let result1 = md5(&[Value::String("test".to_string())]).unwrap();
        let result2 = md5(&[Value::String("test".to_string())]).unwrap();
        assert_eq!(result1, result2);

        let result1 = sha256(&[Value::String("test".to_string())]).unwrap();
        let result2 = sha256(&[Value::String("test".to_string())]).unwrap();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_hash_different_inputs() {
        // Different inputs should produce different hashes
        let hash1 = md5(&[Value::String("test1".to_string())]).unwrap();
        let hash2 = md5(&[Value::String("test2".to_string())]).unwrap();
        assert_ne!(hash1, hash2);
    }
}
