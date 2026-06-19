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
        "_FS" => Some(fs_module()),
        "_TIME" => Some(time_module()),
        _ => None,
    }
}

fn math_module() -> NativeModule {
    let mut functions: HashMap<&'static str, NativeFunctionInfo> = HashMap::new();

    functions.insert(
        "SIN",
        NativeFunctionInfo {
            func: psy_math::sin,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "COS",
        NativeFunctionInfo {
            func: psy_math::cos,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "TAN",
        NativeFunctionInfo {
            func: psy_math::tan,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "SQRT",
        NativeFunctionInfo {
            func: psy_math::sqrt,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "POW",
        NativeFunctionInfo {
            func: psy_math::pow,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "ABS",
        NativeFunctionInfo {
            func: psy_math::abs,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ROUND",
        NativeFunctionInfo {
            func: psy_math::round,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "FLOOR",
        NativeFunctionInfo {
            func: psy_math::floor,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "CEIL",
        NativeFunctionInfo {
            func: psy_math::ceil,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "MEAN",
        NativeFunctionInfo {
            func: psy_math::mean,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MEDIAN",
        NativeFunctionInfo {
            func: psy_math::median,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MODE",
        NativeFunctionInfo {
            func: psy_math::mode,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "VARIANCE",
        NativeFunctionInfo {
            func: psy_math::variance,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "STDDEV",
        NativeFunctionInfo {
            func: psy_math::stddev,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MIN",
        NativeFunctionInfo {
            func: psy_math::min,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MAX",
        NativeFunctionInfo {
            func: psy_math::max,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "SUM",
        NativeFunctionInfo {
            func: psy_math::sum,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "PRODUCT",
        NativeFunctionInfo {
            func: psy_math::product,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "GCD",
        NativeFunctionInfo {
            func: psy_math::gcd,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "LCM",
        NativeFunctionInfo {
            func: psy_math::lcm,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "IS_PRIME",
        NativeFunctionInfo {
            func: psy_math::is_prime,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "MATRIX_ADD",
        NativeFunctionInfo {
            func: psy_math::matrix_add,
            arity: Arity::AtLeast(2),
        },
    );
    functions.insert(
        "MATRIX_MULTIPLY",
        NativeFunctionInfo {
            func: psy_math::matrix_multiply,
            arity: Arity::AtLeast(2),
        },
    );
    functions.insert(
        "MATRIX_TRANSPOSE",
        NativeFunctionInfo {
            func: psy_math::matrix_transpose,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MATRIX_DETERMINANT",
        NativeFunctionInfo {
            func: psy_math::matrix_determinant,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MATRIX_INVERSE",
        NativeFunctionInfo {
            func: psy_math::matrix_inverse,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "DOT",
        NativeFunctionInfo {
            func: psy_math::dot,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "CROSS",
        NativeFunctionInfo {
            func: psy_math::cross,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "ASIN",
        NativeFunctionInfo {
            func: psy_math::asin,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ACOS",
        NativeFunctionInfo {
            func: psy_math::acos,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ATAN",
        NativeFunctionInfo {
            func: psy_math::atan,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "LOG",
        NativeFunctionInfo {
            func: psy_math::log,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "LOG10",
        NativeFunctionInfo {
            func: psy_math::log10,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "EXP",
        NativeFunctionInfo {
            func: psy_math::exp,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "FACTORIAL",
        NativeFunctionInfo {
            func: psy_math::factorial,
            arity: Arity::Exact(1),
        },
    );

    let mut constants: HashMap<&'static str, Value> = HashMap::new();
    constants.insert("PI", psy_math::pi());
    constants.insert("E", psy_math::e());

    NativeModule {
        functions,
        constants,
    }
}

fn fs_module() -> NativeModule {
    let mut functions: HashMap<&'static str, NativeFunctionInfo> = HashMap::new();

    functions.insert(
        "READFILE",
        NativeFunctionInfo {
            func: psy_fs::read_file,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "WRITEFILE",
        NativeFunctionInfo {
            func: psy_fs::write_file,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "EXISTS",
        NativeFunctionInfo {
            func: psy_fs::exists,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ISFILE",
        NativeFunctionInfo {
            func: psy_fs::is_file,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ISDIR",
        NativeFunctionInfo {
            func: psy_fs::is_dir,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "DELETE",
        NativeFunctionInfo {
            func: psy_fs::delete,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "LISTDIR",
        NativeFunctionInfo {
            func: psy_fs::list_dir,
            arity: Arity::Exact(1),
        },
    );

    NativeModule {
        functions,
        constants: HashMap::new(),
    }
}

fn time_module() -> NativeModule {
    let mut functions: HashMap<&'static str, NativeFunctionInfo> = HashMap::new();

    functions.insert(
        "NOW",
        NativeFunctionInfo {
            func: psy_time::now,
            arity: Arity::Exact(0),
        },
    );
    functions.insert(
        "NOWMS",
        NativeFunctionInfo {
            func: psy_time::now_ms,
            arity: Arity::Exact(0),
        },
    );
    functions.insert(
        "SLEEP",
        NativeFunctionInfo {
            func: psy_time::sleep,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "SLEEPMS",
        NativeFunctionInfo {
            func: psy_time::sleep_ms,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "FORMATTIME",
        NativeFunctionInfo {
            func: psy_time::format_time,
            arity: Arity::Exact(1),
        },
    );

    NativeModule {
        functions,
        constants: HashMap::new(),
    }
}
