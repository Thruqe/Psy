use crate::interpreter::environment::Value;
use std::collections::HashMap;

pub type NativeFn = fn(&[Value]) -> Result<Value, String>;

#[derive(Debug, Clone, Copy)]
pub enum Arity {
    Exact(usize),
    AtLeast(usize),
}

impl Arity {
    fn describe(&self) -> String {
        match self {
            Arity::Exact(1) => "1 arg".to_string(),
            Arity::Exact(n) => format!("{} args", n),
            Arity::AtLeast(1) => "1+ args".to_string(),
            Arity::AtLeast(n) => format!("{}+ args", n),
        }
    }
}

#[derive(Clone, Copy)]
pub struct NativeFunctionInfo {
    pub func: NativeFn,
    pub arity: Arity,
}

pub struct NativeModule {
    pub functions: HashMap<&'static str, NativeFunctionInfo>,
    pub constants: HashMap<&'static str, Value>,
}

impl NativeModule {
    /// Produces a human-readable description of every function and
    /// constant this module exposes, for OUTPUT _MATH-style introspection.
    pub fn describe(&self) -> Vec<Value> {
        let mut entries: Vec<Value> = self
            .functions
            .iter()
            .map(|(name, info)| Value::String(format!("{}({})", name, info.arity.describe())))
            .collect();

        entries.extend(
            self.constants
                .keys()
                .map(|name| Value::String(name.to_string())),
        );

        entries.sort_by(|a, b| match (a, b) {
            (Value::String(a), Value::String(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        });

        entries
    }
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
    let mut functions: HashMap<&'static str, NativeFunctionInfo> = HashMap::new();

    functions.insert(
        "SIN",
        NativeFunctionInfo {
            func: pseudocode_math::sin,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "COS",
        NativeFunctionInfo {
            func: pseudocode_math::cos,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "TAN",
        NativeFunctionInfo {
            func: pseudocode_math::tan,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "SQRT",
        NativeFunctionInfo {
            func: pseudocode_math::sqrt,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "POW",
        NativeFunctionInfo {
            func: pseudocode_math::pow,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "ABS",
        NativeFunctionInfo {
            func: pseudocode_math::abs,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ROUND",
        NativeFunctionInfo {
            func: pseudocode_math::round,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "FLOOR",
        NativeFunctionInfo {
            func: pseudocode_math::floor,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "CEIL",
        NativeFunctionInfo {
            func: pseudocode_math::ceil,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "MEAN",
        NativeFunctionInfo {
            func: pseudocode_math::mean,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MEDIAN",
        NativeFunctionInfo {
            func: pseudocode_math::median,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MODE",
        NativeFunctionInfo {
            func: pseudocode_math::mode,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "VARIANCE",
        NativeFunctionInfo {
            func: pseudocode_math::variance,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "STDDEV",
        NativeFunctionInfo {
            func: pseudocode_math::stddev,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MIN",
        NativeFunctionInfo {
            func: pseudocode_math::min,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MAX",
        NativeFunctionInfo {
            func: pseudocode_math::max,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "SUM",
        NativeFunctionInfo {
            func: pseudocode_math::sum,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "PRODUCT",
        NativeFunctionInfo {
            func: pseudocode_math::product,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "GCD",
        NativeFunctionInfo {
            func: pseudocode_math::gcd,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "LCM",
        NativeFunctionInfo {
            func: pseudocode_math::lcm,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "IS_PRIME",
        NativeFunctionInfo {
            func: pseudocode_math::is_prime,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "MATRIX_ADD",
        NativeFunctionInfo {
            func: pseudocode_math::matrix_add,
            arity: Arity::AtLeast(2),
        },
    );
    functions.insert(
        "MATRIX_MULTIPLY",
        NativeFunctionInfo {
            func: pseudocode_math::matrix_multiply,
            arity: Arity::AtLeast(2),
        },
    );
    functions.insert(
        "MATRIX_TRANSPOSE",
        NativeFunctionInfo {
            func: pseudocode_math::matrix_transpose,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MATRIX_DETERMINANT",
        NativeFunctionInfo {
            func: pseudocode_math::matrix_determinant,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MATRIX_INVERSE",
        NativeFunctionInfo {
            func: pseudocode_math::matrix_inverse,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "DOT",
        NativeFunctionInfo {
            func: pseudocode_math::dot,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "CROSS",
        NativeFunctionInfo {
            func: pseudocode_math::cross,
            arity: Arity::Exact(2),
        },
    );

    let mut constants: HashMap<&'static str, Value> = HashMap::new();
    constants.insert("PI", pseudocode_math::pi());
    constants.insert("E", pseudocode_math::e());

    NativeModule {
        functions,
        constants,
    }
}
