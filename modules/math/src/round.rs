use pseudocode_types::Value;

pub fn round(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "ROUND")?;
    Ok(Value::Number(n.round()))
}