use helper::Value;

pub fn pi() -> Value {
    Value::Number(std::f64::consts::PI)
}

pub fn e() -> Value {
    Value::Number(std::f64::consts::E)
}
