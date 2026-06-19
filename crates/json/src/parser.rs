use types::Value;

pub fn parse(s: &str) -> Result<Value, String> {
    let s = s.trim();

    if s.is_empty() {
        return Ok(Value::String("".to_string()));
    }

    match s.chars().next().unwrap() {
        '"' => parse_string(s),
        '[' => parse_array(s),
        '{' => parse_object(s),
        't' | 'f' => parse_boolean(s),
        'n' => parse_null(s),
        '-' | '0'..='9' => parse_number(s),
        _ => Err(format!(
            "Unexpected character: {}",
            s.chars().next().unwrap()
        )),
    }
}

fn parse_string(s: &str) -> Result<Value, String> {
    // Remove surrounding quotes
    if !s.starts_with('"') || !s.ends_with('"') {
        return Err("Invalid string format".to_string());
    }

    let inner = &s[1..s.len() - 1];
    let mut result = String::new();
    let mut chars = inner.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('u') => {
                    // Unicode escape - simplified: just skip for now
                    for _ in 0..4 {
                        chars.next();
                    }
                    result.push('?');
                }
                Some(c) => result.push(c),
                None => break,
            }
        } else {
            result.push(c);
        }
    }

    Ok(Value::String(result))
}

fn parse_number(s: &str) -> Result<Value, String> {
    match s.parse::<f64>() {
        Ok(n) => Ok(Value::Number(n)),
        Err(_) => Err(format!("Invalid number: {}", s)),
    }
}

fn parse_boolean(s: &str) -> Result<Value, String> {
    match s {
        "true" => Ok(Value::Boolean(true)),
        "false" => Ok(Value::Boolean(false)),
        _ => Err(format!("Invalid boolean: {}", s)),
    }
}

fn parse_null(s: &str) -> Result<Value, String> {
    match s {
        "null" => Ok(Value::String("null".to_string())),
        _ => Err(format!("Invalid null: {}", s)),
    }
}

fn parse_array(s: &str) -> Result<Value, String> {
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err("Invalid array format".to_string());
    }

    let inner = &s[1..s.len() - 1];
    let items = split_json(inner);
    let mut values = Vec::new();

    for item in items {
        let trimmed = item.trim();
        if !trimmed.is_empty() {
            values.push(parse(trimmed)?);
        }
    }

    Ok(Value::Array(values))
}

fn parse_object(s: &str) -> Result<Value, String> {
    if !s.starts_with('{') || !s.ends_with('}') {
        return Err("Invalid object format".to_string());
    }

    let inner = &s[1..s.len() - 1];
    let pairs = split_json(inner);
    let mut key_values = Vec::new();

    for pair in pairs {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }

        // Find the colon that separates key and value
        if let Some(pos) = find_colon(pair) {
            let key = pair[..pos].trim();
            let value = pair[pos + 1..].trim();

            let parsed_key = parse(key)?;
            let parsed_value = parse(value)?;

            let key_str = value_to_string(&parsed_key);
            let value_str = value_to_string(&parsed_value);

            key_values.push(Value::String(format!("{}: {}", key_str, value_str)));
        }
    }

    Ok(Value::Array(key_values))
}

fn find_colon(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut prev_char = ' ';

    for (i, c) in s.char_indices() {
        match c {
            '"' if prev_char != '\\' => in_string = !in_string,
            '[' | '{' if !in_string => depth += 1,
            ']' | '}' if !in_string => depth -= 1,
            ':' if depth == 0 && !in_string => return Some(i),
            _ => {}
        }
        prev_char = c;
    }

    None
}

fn split_json(s: &str) -> Vec<String> {
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
                items.push(current.clone());
                current = String::new();
                continue;
            }
            _ => {}
        }
        current.push(c);
        prev_char = c;
    }

    if !current.trim().is_empty() {
        items.push(current);
    }

    items
}

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
