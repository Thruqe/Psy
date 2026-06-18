use psy_types::Value;

pub fn tan(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "TAN")?;
    // Convert degrees to radians
    let radians = n.to_radians();
    Ok(Value::Number(radians.tan()))
}
