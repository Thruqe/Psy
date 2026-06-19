pub mod parser;
pub mod serializer;

use types::Value;

pub fn json_parse(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("JSON_PARSE expects 1 argument (json string)".to_string());
    }

    let json_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("JSON_PARSE expects a string".to_string()),
    };

    parser::parse(json_str.trim())
}

pub fn json_stringify(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("JSON_STRINGIFY expects 1 argument (value)".to_string());
    }

    Ok(Value::String(serializer::stringify(&args[0])))
}

pub fn json_get(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("JSON_GET expects 2 arguments (json_string, key)".to_string());
    }

    let json_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("JSON_GET expects a string as first argument".to_string()),
    };

    let key = match &args[1] {
        Value::String(s) => s.clone(),
        Value::Number(n) => format!("{}", n),
        _ => return Err("JSON_GET expects a string or number as key".to_string()),
    };

    let parsed = parser::parse(&json_str)?;

    match &parsed {
        Value::Array(arr) => {
            // Try numeric index first
            if let Ok(index) = key.parse::<usize>() {
                return arr
                    .get(index)
                    .cloned()
                    .ok_or_else(|| format!("Index {} out of bounds", index));
            }

            // Try string key (for objects represented as arrays of "key: value" strings)
            for item in arr {
                if let Value::String(s) = item {
                    if let Some(colon_pos) = s.find(':') {
                        let item_key = s[..colon_pos].trim();
                        if item_key == key {
                            let value_str = s[colon_pos + 1..].trim();
                            // Try to parse the value as JSON
                            if value_str.starts_with('[') || value_str.starts_with('{') {
                                return parser::parse(value_str);
                            } else {
                                return Ok(Value::String(value_str.to_string()));
                            }
                        }
                    }
                }
            }

            // Check for nested objects
            for item in arr {
                if let Value::String(s) = item {
                    if let Some(colon_pos) = s.find(':') {
                        let item_key = s[..colon_pos].trim();
                        let value_str = s[colon_pos + 1..].trim();

                        // If value is an array/object, parse it and search recursively
                        if value_str.starts_with('[') || value_str.starts_with('{') {
                            if let Ok(nested) = parser::parse(value_str) {
                                if let Value::Array(nested_arr) = nested {
                                    // Try to find the key in the nested array
                                    for nested_item in &nested_arr {
                                        if let Value::String(ns) = nested_item {
                                            if let Some(nested_colon) = ns.find(':') {
                                                let n_key = ns[..nested_colon].trim();
                                                if n_key == key {
                                                    let n_value = ns[nested_colon + 1..].trim();
                                                    if n_value.starts_with('[')
                                                        || n_value.starts_with('{')
                                                    {
                                                        return parser::parse(n_value);
                                                    } else {
                                                        return Ok(Value::String(
                                                            n_value.to_string(),
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Err(format!("Key '{}' not found in JSON object", key))
        }
        _ => Err("JSON_GET requires a parsed JSON array or object".to_string()),
    }
}

pub fn json_set(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 {
        return Err("JSON_SET expects 3 arguments (json_string, key, value)".to_string());
    }

    let json_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("JSON_SET expects a string as first argument".to_string()),
    };

    let key = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err("JSON_SET expects a string key".to_string()),
    };

    let value = &args[2];

    let mut parsed = parser::parse(&json_str)?;

    if let Value::Array(ref mut arr) = parsed {
        // Try to update existing key
        for item in arr.iter_mut() {
            if let Value::String(s) = item {
                if let Some(colon_pos) = s.find(':') {
                    let item_key = s[..colon_pos].trim();
                    if item_key == key {
                        *item = Value::String(format!("{}: {}", key, serializer::stringify(value)));
                        return Ok(Value::String(serializer::stringify(&Value::Array(
                            arr.clone(),
                        ))));
                    }
                }
            }
        }

        // Key not found, add new key-value pair
        arr.push(Value::String(format!(
            "{}: {}",
            key,
            serializer::stringify(value)
        )));
        Ok(Value::String(serializer::stringify(&Value::Array(
            arr.clone(),
        ))))
    } else {
        Err("JSON_SET currently only supports object modification".to_string())
    }
}

pub fn json_keys(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("JSON_KEYS expects 1 argument (json_string)".to_string());
    }

    let json_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("JSON_KEYS expects a string".to_string()),
    };

    let parsed = parser::parse(&json_str)?;
    let keys = get_keys(&parsed);
    Ok(Value::Array(keys))
}

fn get_keys(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(arr) => {
            let mut keys = Vec::new();
            for item in arr {
                if let Value::String(s) = item {
                    if let Some(pos) = s.find(':') {
                        keys.push(Value::String(s[..pos].trim().to_string()));
                    }
                }
            }
            keys
        }
        _ => Vec::new(),
    }
}
