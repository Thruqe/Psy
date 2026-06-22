use helper::Value;

pub fn log(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "LOG")?;
    if n <= 0.0 {
        return Err("LOG expects a positive number".to_string());
    }
    Ok(Value::Number(n.ln()))
}

pub fn log10(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "LOG10")?;
    if n <= 0.0 {
        return Err("LOG10 expects a positive number".to_string());
    }
    Ok(Value::Number(n.log10()))
}
