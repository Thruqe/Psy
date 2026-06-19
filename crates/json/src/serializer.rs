use types::Value;

pub fn stringify(value: &Value) -> String {
    match value {
        Value::String(s) => {
            // Check if it's already a JSON string
            if is_json(s) {
                s.clone()
            } else {
                format!("\"{}\"", escape_string(s))
            }
        }
        Value::Number(n) => format!("{}", n),
        Value::Boolean(b) => format!("{}", b),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(|v| stringify(v)).collect();
            format!("[{}]", elements.join(","))
        }
        Value::Undefined => "null".to_string(),
    }
}

fn escape_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            c => c.to_string(),
        })
        .collect()
}

fn is_json(s: &str) -> bool {
    let trimmed = s.trim();
    trimmed.starts_with('{')
        || trimmed.starts_with('[')
        || trimmed == "null"
        || trimmed == "true"
        || trimmed == "false"
}
