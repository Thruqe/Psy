use pseudocode_types::Value;

pub fn stddev(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("STDDEV expects at least 1 argument".to_string());
    }

    let mut numbers = Vec::new();

    for arg in args {
        match arg {
            Value::Number(n) => numbers.push(*n),
            Value::Array(arr) => {
                for element in arr {
                    match element {
                        Value::Number(n) => numbers.push(*n),
                        _ => return Err("STDDEV expects numbers or arrays of numbers".to_string()),
                    }
                }
            }
            _ => return Err("STDDEV expects numbers or arrays of numbers".to_string()),
        }
    }

    if numbers.is_empty() {
        return Err("STDDEV requires at least one number".to_string());
    }

    // Calculate mean
    let sum: f64 = numbers.iter().sum();
    let mean = sum / numbers.len() as f64;

    // Calculate variance
    let sum_sq_diff: f64 = numbers.iter().map(|&x| (x - mean).powi(2)).sum();
    let variance = sum_sq_diff / numbers.len() as f64;

    // Calculate standard deviation
    let stddev = variance.sqrt();

    Ok(Value::Number(stddev))
}
