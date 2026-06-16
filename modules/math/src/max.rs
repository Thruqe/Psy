use pseudocode_types::Value;

pub fn max(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("MAX expects at least 1 argument".to_string());
    }

    let mut max_val = f64::NEG_INFINITY;
    let mut found = false;

    for arg in args {
        match arg {
            Value::Number(n) => {
                if *n > max_val {
                    max_val = *n;
                }
                found = true;
            }
            Value::Array(arr) => {
                for element in arr {
                    match element {
                        Value::Number(n) => {
                            if *n > max_val {
                                max_val = *n;
                            }
                            found = true;
                        }
                        _ => return Err("MAX expects numbers or arrays of numbers".to_string()),
                    }
                }
            }
            _ => return Err("MAX expects numbers or arrays of numbers".to_string()),
        }
    }

    if !found {
        return Err("MAX requires at least one number".to_string());
    }

    Ok(Value::Number(max_val))
}
