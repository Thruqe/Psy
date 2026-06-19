use types::Value;

pub fn gcd(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("GCD expects 2 arguments".to_string());
    }
    let a = super::expect_number(&args[0], "GCD")?;
    let b = super::expect_number(&args[1], "GCD")?;

    if a.fract() != 0.0 || b.fract() != 0.0 {
        return Err("GCD expects whole numbers".to_string());
    }

    let mut a = a.abs() as i64;
    let mut b = b.abs() as i64;
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    Ok(Value::Number(a as f64))
}

pub fn lcm(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("LCM expects 2 arguments".to_string());
    }
    let a = super::expect_number(&args[0], "LCM")?;
    let b = super::expect_number(&args[1], "LCM")?;

    if a.fract() != 0.0 || b.fract() != 0.0 {
        return Err("LCM expects whole numbers".to_string());
    }

    let a = a.abs() as i64;
    let b = b.abs() as i64;

    if a == 0 || b == 0 {
        return Ok(Value::Number(0.0));
    }

    let gcd_val = {
        let (mut x, mut y) = (a, b);
        while y != 0 {
            let t = y;
            y = x % y;
            x = t;
        }
        x
    };

    Ok(Value::Number(((a / gcd_val) * b) as f64))
}

pub fn is_prime(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("IS_PRIME expects 1 argument".to_string());
    }
    let n = super::expect_number(&args[0], "IS_PRIME")?;

    if n.fract() != 0.0 {
        return Err("IS_PRIME expects a whole number".to_string());
    }

    let n = n as i64;
    if n < 2 {
        return Ok(Value::Boolean(false));
    }
    if n == 2 {
        return Ok(Value::Boolean(true));
    }
    if n % 2 == 0 {
        return Ok(Value::Boolean(false));
    }

    let mut i = 3i64;
    while i * i <= n {
        if n % i == 0 {
            return Ok(Value::Boolean(false));
        }
        i += 2;
    }
    Ok(Value::Boolean(true))
}
