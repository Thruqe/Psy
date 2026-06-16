use pseudocode_types::Value;

pub fn abs(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "ABS")?;
    Ok(Value::Number(n.abs()))
}