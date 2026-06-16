use pseudocode_types::Value;

mod abs;
mod ceil;
mod constants;
mod cos;
mod floor;
mod pow;
mod round;
mod sin;
mod sqrt;
mod tan;

pub use abs::abs;
pub use ceil::ceil;
pub use constants::{e, pi};
pub use cos::cos;
pub use floor::floor;
pub use pow::pow;
pub use round::round;
pub use sin::sin;
pub use sqrt::sqrt;
pub use tan::tan;

pub(crate) fn expect_number(value: &Value, fn_name: &str) -> Result<f64, String> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(format!("{} expects a number argument", fn_name)),
    }
}

pub(crate) fn expect_one_number(args: &[Value], fn_name: &str) -> Result<f64, String> {
    if args.len() != 1 {
        return Err(format!("{} expects exactly 1 argument", fn_name));
    }
    expect_number(&args[0], fn_name)
}
