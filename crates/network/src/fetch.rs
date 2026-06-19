use super::http;
use types::Value;

pub fn fetch_url(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("FETCH expects at least 1 argument (url)".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("FETCH expects a string URL".to_string()),
    };

    // Simply delegate to HTTP GET for now
    http::get(&[Value::String(url)])
}

pub fn url_encode(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("URL_ENCODE expects 1 argument (text)".to_string());
    }

    let text = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("URL_ENCODE expects a string".to_string()),
    };

    let encoded: String = text
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect();

    Ok(Value::String(encoded))
}

pub fn url_decode(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("URL_DECODE expects 1 argument (text)".to_string());
    }

    let text = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("URL_DECODE expects a string".to_string()),
    };

    let decoded =
        text.replace('+', " ")
            .split('%')
            .enumerate()
            .fold(String::new(), |mut acc, (i, part)| {
                if i == 0 {
                    acc.push_str(part);
                } else if part.len() >= 2 {
                    if let Ok(byte) = u8::from_str_radix(&part[..2], 16) {
                        acc.push(byte as char);
                        acc.push_str(&part[2..]);
                    }
                }
                acc
            });

    Ok(Value::String(decoded))
}

pub fn parse_json(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("PARSE_JSON expects 1 argument (json string)".to_string());
    }

    let json_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("PARSE_JSON expects a string".to_string()),
    };

    // Simple JSON parser for basic types
    parse_json_value(json_str.trim())
}

fn parse_json_value(s: &str) -> Result<Value, String> {
    let s = s.trim();

    if s.is_empty() {
        return Ok(Value::String("".to_string()));
    }

    match s.chars().next().unwrap() {
        '"' => {
            // String
            let content = s[1..s.len() - 1].to_string();
            Ok(Value::String(content))
        }
        '[' => {
            // Array
            let content = &s[1..s.len() - 1];
            let mut values = Vec::new();

            for item in split_json_array(content) {
                values.push(parse_json_value(&item)?);
            }

            Ok(Value::Array(values))
        }
        '{' => {
            // Object - simplified to array of key-value pairs
            let content = &s[1..s.len() - 1];
            let mut pairs = Vec::new();

            for pair in split_json_array(content) {
                if let Some(pos) = pair.find(':') {
                    let key = pair[..pos].trim();
                    let value = pair[pos + 1..].trim();
                    let key_str = value_to_string(&parse_json_value(key)?);
                    let value_str = value_to_string(&parse_json_value(value)?);
                    pairs.push(Value::String(format!("{}: {}", key_str, value_str)));
                }
            }

            Ok(Value::Array(pairs))
        }
        't' if s == "true" => Ok(Value::Boolean(true)),
        'f' if s == "false" => Ok(Value::Boolean(false)),
        'n' if s == "null" => Ok(Value::String("null".to_string())),
        _ => {
            // Number
            if let Ok(n) = s.parse::<f64>() {
                Ok(Value::Number(n))
            } else {
                Err(format!("Invalid JSON value: {}", s))
            }
        }
    }
}

fn split_json_array(s: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    let mut in_string = false;
    let mut prev_char = ' ';

    for c in s.chars() {
        match c {
            '"' if prev_char != '\\' => in_string = !in_string,
            '[' | '{' if !in_string => depth += 1,
            ']' | '}' if !in_string => depth -= 1,
            ',' if depth == 0 && !in_string => {
                items.push(current.trim().to_string());
                current = String::new();
                continue;
            }
            _ => {}
        }
        current.push(c);
        prev_char = c;
    }

    if !current.trim().is_empty() {
        items.push(current.trim().to_string());
    }

    items
}

// Helper function to convert Value to string (instead of Display trait)
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Number(n) => format!("{}", n),
        Value::String(s) => s.clone(),
        Value::Boolean(b) => format!("{}", b),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(|v| value_to_string(v)).collect();
            format!("[{}]", elements.join(", "))
        }
        Value::Undefined => "undefined".to_string(),
    }
}
