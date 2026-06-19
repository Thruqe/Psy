use types::Value;

pub fn min(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("MIN expects at least 1 argument".to_string());
    }

    let mut min_val = f64::INFINITY;
    let mut found = false;

    for arg in args {
        match arg {
            Value::Number(n) => {
                if *n < min_val {
                    min_val = *n;
                }
                found = true;
            }
            Value::Array(arr) => {
                for element in arr {
                    match element {
                        Value::Number(n) => {
                            if *n < min_val {
                                min_val = *n;
                            }
                            found = true;
                        }
                        _ => return Err("MIN expects numbers or arrays of numbers".to_string()),
                    }
                }
            }
            _ => return Err("MIN expects numbers or arrays of numbers".to_string()),
        }
    }

    if !found {
        return Err("MIN requires at least one number".to_string());
    }

    Ok(Value::Number(min_val))
}
