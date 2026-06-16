use pseudocode_types::Value;

pub fn product(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("PRODUCT expects at least 1 argument".to_string());
    }

    let mut total = 1.0;
    let mut found = false;

    for arg in args {
        match arg {
            Value::Number(n) => {
                total *= n;
                found = true;
            }
            Value::Array(arr) => {
                for element in arr {
                    match element {
                        Value::Number(n) => {
                            total *= n;
                            found = true;
                        }
                        _ => return Err("PRODUCT expects numbers or arrays of numbers".to_string()),
                    }
                }
            }
            _ => return Err("PRODUCT expects numbers or arrays of numbers".to_string()),
        }
    }

    if !found {
        return Err("PRODUCT requires at least one number".to_string());
    }

    Ok(Value::Number(total))
}
