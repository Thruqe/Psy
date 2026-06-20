use types::Value;

/// OS_EXIT(code?) → never returns
pub fn os_exit(args: &[Value]) -> Result<Value, String> {
    let code = if args.is_empty() {
        0
    } else {
        match &args[0] {
            Value::Number(n) => *n as i32,
            _ => 0,
        }
    };
    std::process::exit(code);
}
