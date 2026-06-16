use crate::interpreter::environment::Value;

pub type NativeFn = fn(&[Value]) -> Result<Value, String>;

pub struct NativeModule {
    pub functions: std::collections::HashMap<&'static str, NativeFn>,
    pub constants: std::collections::HashMap<&'static str, Value>,
}

/// Returns the native module registry entry for `module_name`, or None
/// if it isn't a recognized built-in module.
pub fn get_module(module_name: &str) -> Option<NativeModule> {
    match module_name {
        "_MATH" => Some(math_module()),
        _ => None,
    }
}

fn math_module() -> NativeModule {
    let mut functions: std::collections::HashMap<&'static str, NativeFn> =
        std::collections::HashMap::new();
    functions.insert("SIN", pseudocode_math::sin);
    functions.insert("COS", pseudocode_math::cos);
    functions.insert("TAN", pseudocode_math::tan);
    functions.insert("SQRT", pseudocode_math::sqrt);
    functions.insert("POW", pseudocode_math::pow);
    functions.insert("ABS", pseudocode_math::abs);
    functions.insert("ROUND", pseudocode_math::round);
    functions.insert("FLOOR", pseudocode_math::floor);
    functions.insert("CEIL", pseudocode_math::ceil);
    functions.insert("MEAN", pseudocode_math::mean);
    functions.insert("MEDIAN", pseudocode_math::median);
    functions.insert("MODE", pseudocode_math::mode);
    functions.insert("VARIANCE", pseudocode_math::variance);
    functions.insert("STDDEV", pseudocode_math::stddev);
    functions.insert("MIN", pseudocode_math::min);
    functions.insert("MAX", pseudocode_math::max);
    functions.insert("SUM", pseudocode_math::sum);
    functions.insert("PRODUCT", pseudocode_math::product);

    let mut constants: std::collections::HashMap<&'static str, Value> =
        std::collections::HashMap::new();
    constants.insert("PI", pseudocode_math::pi());
    constants.insert("E", pseudocode_math::e());

    NativeModule {
        functions,
        constants,
    }
}
