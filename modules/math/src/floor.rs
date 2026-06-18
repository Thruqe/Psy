use psy_types::Value;

pub fn floor(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "FLOOR")?;
    Ok(Value::Number(n.floor()))
}
