use pseudocode_types::Value;

pub fn mean(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("MEAN expects at least 1 argument".to_string());
    }

    let mut sum = 0.0;
    let mut count = 0;

    for arg in args {
        match arg {
            Value::Number(n) => {
                sum += n;
                count += 1;
            }
            Value::Array(arr) => {
                for element in arr {
                    match element {
                        Value::Number(n) => {
                            sum += n;
                            count += 1;
                        }
                        _ => return Err("MEAN expects numbers or arrays of numbers".to_string()),
                    }
                }
            }
            _ => return Err("MEAN expects numbers or arrays of numbers".to_string()),
        }
    }

    if count == 0 {
        return Err("MEAN requires at least one number".to_string());
    }

    Ok(Value::Number(sum / count as f64))
}
