use types::Value;

pub fn sin(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "SIN")?;
    Ok(Value::Number(n.sin()))
}
