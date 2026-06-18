use psy_types::Value;

pub fn cos(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "COS")?;
    // Convert degrees to radians
    let radians = n.to_radians();
    Ok(Value::Number(radians.cos()))
}
