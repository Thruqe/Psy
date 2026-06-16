use pseudocode_types::Value;

pub fn pow(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("POW expects 2 arguments".to_string());
    }
    let base = super::expect_number(&args[0], "POW")?;
    let exp = super::expect_number(&args[1], "POW")?;
    Ok(Value::Number(base.powf(exp)))
}