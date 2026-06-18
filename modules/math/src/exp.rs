use psy_types::Value;

pub fn exp(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "EXP")?;
    Ok(Value::Number(n.exp()))
}
