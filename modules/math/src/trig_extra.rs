use psy_types::Value;

pub fn asin(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "ASIN")?;
    if n < -1.0 || n > 1.0 {
        return Err("ASIN expects a value between -1 and 1".to_string());
    }
    Ok(Value::Number(n.asin()))
}

pub fn acos(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "ACOS")?;
    if n < -1.0 || n > 1.0 {
        return Err("ACOS expects a value between -1 and 1".to_string());
    }
    Ok(Value::Number(n.acos()))
}

pub fn atan(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "ATAN")?;
    Ok(Value::Number(n.atan()))
}
