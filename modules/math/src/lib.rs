mod abs;
mod algebra;
mod ceil;
mod constants;
mod cos;
mod floor;
mod matrix;
mod matrix_ops;
mod max;
mod mean;
mod median;
mod min;
mod mode;
mod pow;
mod product;
mod round;
mod sin;
mod sqrt;
mod stddev;
mod sum;
mod tan;
mod variance;
mod vector_ops;

pub use abs::abs;
pub use algebra::{gcd, is_prime, lcm};
pub use ceil::ceil;
pub use constants::e;
pub use constants::pi;
pub use cos::cos;
pub use floor::floor;
pub use matrix_ops::{
    matrix_add, matrix_determinant, matrix_inverse, matrix_multiply, matrix_transpose,
};
pub use max::max;
pub use mean::mean;
pub use median::median;
pub use min::min;
pub use mode::mode;
pub use pow::pow;
pub use product::product;
pub use round::round;
pub use sin::sin;
pub use sqrt::sqrt;
pub use stddev::stddev;
pub use sum::sum;
pub use tan::tan;
pub use variance::variance;
pub use vector_ops::{cross, dot};

use pseudocode_types::Value;

pub(crate) fn expect_number(value: &Value, fn_name: &str) -> Result<f64, String> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(format!("{} expects a number argument", fn_name)),
    }
}

fn expect_one_number(args: &[Value], name: &str) -> Result<f64, String> {
    if args.len() != 1 {
        return Err(format!("{} expects 1 argument", name));
    }
    match args[0] {
        Value::Number(n) => Ok(n),
        _ => Err(format!("{} expects a number", name)),
    }
}
