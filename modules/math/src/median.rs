use psy_types::Value;

pub fn median(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("MEDIAN expects at least 1 argument".to_string());
    }

    let mut numbers = Vec::new();

    for arg in args {
        match arg {
            Value::Number(n) => numbers.push(*n),
            Value::Array(arr) => {
                for element in arr {
                    match element {
                        Value::Number(n) => numbers.push(*n),
                        _ => return Err("MEDIAN expects numbers or arrays of numbers".to_string()),
                    }
                }
            }
            _ => return Err("MEDIAN expects numbers or arrays of numbers".to_string()),
        }
    }

    if numbers.is_empty() {
        return Err("MEDIAN requires at least one number".to_string());
    }

    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let len = numbers.len();

    if len % 2 == 0 {
        // Even number of elements - average of two middle values
        let mid = len / 2;
        let median = (numbers[mid - 1] + numbers[mid]) / 2.0;
        Ok(Value::Number(median))
    } else {
        // Odd number of elements - middle value
        let mid = len / 2;
        Ok(Value::Number(numbers[mid]))
    }
}
