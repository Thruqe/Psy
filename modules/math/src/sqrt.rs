use psy_types::Value;

pub fn sqrt(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "SQRT")?;
    if n < 0.0 {
        return Err("SQRT of a negative number".to_string());
    }
    Ok(Value::Number(n.sqrt()))
}
