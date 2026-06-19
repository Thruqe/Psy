use std::collections::HashMap;
use types::Value;

pub fn mode(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("MODE expects at least 1 argument".to_string());
    }

    let mut numbers = Vec::new();

    for arg in args {
        match arg {
            Value::Number(n) => numbers.push(*n),
            Value::Array(arr) => {
                for element in arr {
                    match element {
                        Value::Number(n) => numbers.push(*n),
                        _ => return Err("MODE expects numbers or arrays of numbers".to_string()),
                    }
                }
            }
            _ => return Err("MODE expects numbers or arrays of numbers".to_string()),
        }
    }

    if numbers.is_empty() {
        return Err("MODE requires at least one number".to_string());
    }

    // Count frequency of each number
    let mut frequency = HashMap::new();
    for &num in &numbers {
        *frequency.entry(num.to_string()).or_insert(0) += 1;
    }

    // Find the maximum frequency
    let max_freq = frequency.values().max().unwrap();

    // Collect all numbers with the maximum frequency
    let mut modes: Vec<f64> = Vec::new();
    for (num_str, &freq) in &frequency {
        if freq == *max_freq {
            if let Ok(num) = num_str.parse::<f64>() {
                modes.push(num);
            }
        }
    }

    // Sort modes for consistent output
    modes.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if modes.len() == 1 {
        Ok(Value::Number(modes[0]))
    } else {
        // Multiple modes - return as array
        let mode_values: Vec<Value> = modes.into_iter().map(Value::Number).collect();
        Ok(Value::Array(mode_values))
    }
}
