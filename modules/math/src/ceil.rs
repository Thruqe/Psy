use pseudocode_types::Value;

pub fn ceil(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "CEIL")?;
    Ok(Value::Number(n.ceil()))
}