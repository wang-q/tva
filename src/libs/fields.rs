use intspan::IntSpan;
use std::collections::HashMap;

pub fn fields_to_ints(s: &str) -> IntSpan {
    let mut ints = IntSpan::new();
    let parts: Vec<&str> = s.split(',').collect();
    for p in parts {
        ints.add_runlist(p);
    }

    ints
}

pub fn parse_numeric_field_list(spec: &str) -> Result<Vec<usize>, String> {
    if spec.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut ints: Vec<i32> = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err(format!("empty field list element in `{}`", spec));
        }
        let intspan = IntSpan::from(part);
        for e in intspan.elements() {
            if e <= 0 {
                return Err(format!("field index must be >= 1 in `{}`", spec));
            }
            ints.push(e);
        }
    }

    ints.sort_unstable();
    ints.dedup();

    Ok(ints.iter().map(|e| *e as usize).collect())
}

pub fn fields_to_idx(spec: &str) -> Vec<usize> {
    parse_numeric_field_list(spec).unwrap()
}

pub fn parse_field_list_with_header(
    spec: &str,
    header: Option<&Header>,
    delimiter: char,
) -> Result<Vec<usize>, String> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut indices: Vec<usize> = Vec::new();

    for part in trimmed.split(',') {
        let token = part.trim();
        if token.is_empty() {
            return Err(format!("empty field list element in `{}`", spec));
        }

        let is_numeric_like = token
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-');

        if is_numeric_like {
            let intspan = IntSpan::from(token);
            for e in intspan.elements() {
                if e <= 0 {
                    return Err(format!("field index must be >= 1 in `{}`", spec));
                }
                indices.push(e as usize);
            }
        } else {
            match header {
                Some(h) => {
                    if let Some(idx0) = h.get_index(token) {
                        indices.push(idx0 + 1);
                    } else {
                        return Err(format!(
                            "unknown field name `{}` in `{}`",
                            token, spec
                        ));
                    }
                }
                None => {
                    return Err(format!(
                        "field name `{}` requires header in `{}`",
                        token, spec
                    ));
                }
            }
        }
    }

    indices.sort_unstable();
    indices.dedup();

    Ok(indices)
}

pub struct Header {
    pub fields: Vec<String>,
    pub index_by_name: HashMap<String, usize>,
}

impl Header {
    pub fn from_fields(fields: Vec<String>) -> Header {
        let mut index_by_name = HashMap::new();
        for (idx, name) in fields.iter().enumerate() {
            index_by_name.entry(name.clone()).or_insert(idx);
        }
        Header {
            fields,
            index_by_name,
        }
    }

    pub fn from_line(line: &str, delimiter: char) -> Header {
        let fields: Vec<String> = if line.is_empty() {
            Vec::new()
        } else {
            line.split(delimiter).map(|s| s.to_string()).collect()
        };
        Header::from_fields(fields)
    }

    pub fn get_index(&self, name: &str) -> Option<usize> {
        self.index_by_name.get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numeric_field_list_basic() {
        let v = parse_numeric_field_list("1,3-5").unwrap();
        assert_eq!(v, vec![1, 3, 4, 5]);
    }

    #[test]
    fn test_parse_numeric_field_list_whitespace_and_dup() {
        let v = parse_numeric_field_list(" 2 , 2 , 4-5 ").unwrap();
        assert_eq!(v, vec![2, 4, 5]);
    }

    #[test]
    fn test_parse_numeric_field_list_empty() {
        let v = parse_numeric_field_list("   ").unwrap();
        assert!(v.is_empty());
    }

    #[test]
    fn parse_field_list_with_header_numeric_and_names() {
        let header = Header::from_line("a\tb\tc", '\t');
        let v = parse_field_list_with_header("1,c", Some(&header), '\t').unwrap();
        assert_eq!(v, vec![1, 3]);
    }

    #[test]
    fn parse_field_list_with_header_unknown_name() {
        let header = Header::from_line("a\tb\tc", '\t');
        let err = parse_field_list_with_header("d", Some(&header), '\t').unwrap_err();
        assert!(err.contains("unknown field name"));
    }

    #[test]
    fn parse_field_list_with_header_requires_header_for_name() {
        let err = parse_field_list_with_header("name", None, '\t').unwrap_err();
        assert!(err.contains("requires header"));
    }

    #[test]
    fn header_from_line_basic() {
        let h = Header::from_line("a\tb\tc", '\t');
        assert_eq!(h.fields, vec!["a", "b", "c"]);
        assert_eq!(h.get_index("a"), Some(0));
        assert_eq!(h.get_index("b"), Some(1));
        assert_eq!(h.get_index("c"), Some(2));
        assert_eq!(h.get_index("d"), None);
    }

    #[test]
    fn header_from_line_empty() {
        let h = Header::from_line("", '\t');
        assert!(h.fields.is_empty());
        assert!(h.index_by_name.is_empty());
    }
}
