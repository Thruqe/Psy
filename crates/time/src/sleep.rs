use std::thread;
use std::time::Duration;
use types::Value;

pub fn sleep(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("SLEEP expects 1 argument (seconds)".to_string());
    }

    let seconds = match &args[0] {
        Value::Number(n) => {
            if *n < 0.0 {
                return Err("SLEEP expects a non-negative number".to_string());
            }
            *n
        }
        _ => return Err("SLEEP expects a number argument (seconds)".to_string()),
    };

    thread::sleep(Duration::from_secs_f64(seconds));
    Ok(Value::Undefined)
}

pub fn sleep_ms(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("SLEEPMS expects 1 argument (milliseconds)".to_string());
    }

    let millis = match &args[0] {
        Value::Number(n) => {
            if *n < 0.0 {
                return Err("SLEEPMS expects a non-negative number".to_string());
            }
            *n
        }
        _ => return Err("SLEEPMS expects a number argument (milliseconds)".to_string()),
    };

    thread::sleep(Duration::from_millis(millis as u64));
    Ok(Value::Undefined)
}
