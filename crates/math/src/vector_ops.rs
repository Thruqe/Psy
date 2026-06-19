use types::Value;

fn extract_vector(value: &Value, fn_name: &str) -> Result<Vec<f64>, String> {
    match value {
        Value::Array(elements) => elements
            .iter()
            .map(|e| match e {
                Value::Number(n) => Ok(*n),
                _ => Err(format!("{}: vector elements must be numbers", fn_name)),
            })
            .collect(),
        _ => Err(format!("{} expects an array (vector) argument", fn_name)),
    }
}

pub fn dot(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("DOT expects 2 arguments".to_string());
    }
    let a = extract_vector(&args[0], "DOT")?;
    let b = extract_vector(&args[1], "DOT")?;

    if a.len() != b.len() {
        return Err("DOT: vectors must be the same length".to_string());
    }

    let result: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    Ok(Value::Number(result))
}

pub fn cross(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("CROSS expects 2 arguments".to_string());
    }
    let a = extract_vector(&args[0], "CROSS")?;
    let b = extract_vector(&args[1], "CROSS")?;

    if a.len() != 3 || b.len() != 3 {
        return Err("CROSS requires exactly 3-element vectors".to_string());
    }

    let result = vec![
        Value::Number(a[1] * b[2] - a[2] * b[1]),
        Value::Number(a[2] * b[0] - a[0] * b[2]),
        Value::Number(a[0] * b[1] - a[1] * b[0]),
    ];
    Ok(Value::Array(result))
}
