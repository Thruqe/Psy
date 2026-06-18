use psy_types::Value;

pub fn factorial(args: &[Value]) -> Result<Value, String> {
    let n = super::expect_one_number(args, "FACTORIAL")?;
    if n < 0.0 || n.fract() != 0.0 {
        return Err("FACTORIAL expects a non-negative whole number".to_string());
    }
    let n = n as u64;
    if n > 20 {
        return Err("FACTORIAL: input too large, result would overflow".to_string());
    }
    let mut result: f64 = 1.0;
    for i in 2..=n {
        result *= i as f64;
    }
    Ok(Value::Number(result))
}
