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
        "_CRYPTO" => Some(crypto_module()),
        _ => None,
    }
}

fn math_module() -> NativeModule {
    let mut functions: HashMap<&'static str, NativeFunctionInfo> = HashMap::new();

    functions.insert(
        "SIN",
        NativeFunctionInfo {
            func: math::sin,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "COS",
        NativeFunctionInfo {
            func: math::cos,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "TAN",
        NativeFunctionInfo {
            func: math::tan,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "SQRT",
        NativeFunctionInfo {
            func: math::sqrt,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "POW",
        NativeFunctionInfo {
            func: math::pow,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "ABS",
        NativeFunctionInfo {
            func: math::abs,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ROUND",
        NativeFunctionInfo {
            func: math::round,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "FLOOR",
        NativeFunctionInfo {
            func: math::floor,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "CEIL",
        NativeFunctionInfo {
            func: math::ceil,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "MEAN",
        NativeFunctionInfo {
            func: math::mean,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MEDIAN",
        NativeFunctionInfo {
            func: math::median,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MODE",
        NativeFunctionInfo {
            func: math::mode,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "VARIANCE",
        NativeFunctionInfo {
            func: math::variance,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "STDDEV",
        NativeFunctionInfo {
            func: math::stddev,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MIN",
        NativeFunctionInfo {
            func: math::min,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MAX",
        NativeFunctionInfo {
            func: math::max,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "SUM",
        NativeFunctionInfo {
            func: math::sum,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "PRODUCT",
        NativeFunctionInfo {
            func: math::product,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "GCD",
        NativeFunctionInfo {
            func: math::gcd,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "LCM",
        NativeFunctionInfo {
            func: math::lcm,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "IS_PRIME",
        NativeFunctionInfo {
            func: math::is_prime,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "MATRIX_ADD",
        NativeFunctionInfo {
            func: math::matrix_add,
            arity: Arity::AtLeast(2),
        },
    );
    functions.insert(
        "MATRIX_MULTIPLY",
        NativeFunctionInfo {
            func: math::matrix_multiply,
            arity: Arity::AtLeast(2),
        },
    );
    functions.insert(
        "MATRIX_TRANSPOSE",
        NativeFunctionInfo {
            func: math::matrix_transpose,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MATRIX_DETERMINANT",
        NativeFunctionInfo {
            func: math::matrix_determinant,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "MATRIX_INVERSE",
        NativeFunctionInfo {
            func: math::matrix_inverse,
            arity: Arity::AtLeast(1),
        },
    );
    functions.insert(
        "DOT",
        NativeFunctionInfo {
            func: math::dot,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "CROSS",
        NativeFunctionInfo {
            func: math::cross,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "ASIN",
        NativeFunctionInfo {
            func: math::asin,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ACOS",
        NativeFunctionInfo {
            func: math::acos,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ATAN",
        NativeFunctionInfo {
            func: math::atan,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "LOG",
        NativeFunctionInfo {
            func: math::log,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "LOG10",
        NativeFunctionInfo {
            func: math::log10,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "EXP",
        NativeFunctionInfo {
            func: math::exp,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "FACTORIAL",
        NativeFunctionInfo {
            func: math::factorial,
            arity: Arity::Exact(1),
        },
    );

    let mut constants: HashMap<&'static str, Value> = HashMap::new();
    constants.insert("PI", math::pi());
    constants.insert("E", math::e());

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
            func: fs::read_file,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "WRITEFILE",
        NativeFunctionInfo {
            func: fs::write_file,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "EXISTS",
        NativeFunctionInfo {
            func: fs::exists,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ISFILE",
        NativeFunctionInfo {
            func: fs::is_file,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "ISDIR",
        NativeFunctionInfo {
            func: fs::is_dir,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "DELETE",
        NativeFunctionInfo {
            func: fs::delete,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "LISTDIR",
        NativeFunctionInfo {
            func: fs::list_dir,
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
            func: time::now,
            arity: Arity::Exact(0),
        },
    );
    functions.insert(
        "NOWMS",
        NativeFunctionInfo {
            func: time::now_ms,
            arity: Arity::Exact(0),
        },
    );
    functions.insert(
        "SLEEP",
        NativeFunctionInfo {
            func: time::sleep,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "SLEEPMS",
        NativeFunctionInfo {
            func: time::sleep_ms,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "FORMATTIME",
        NativeFunctionInfo {
            func: time::format_time,
            arity: Arity::Exact(1),
        },
    );

    NativeModule {
        functions,
        constants: HashMap::new(),
    }
}

fn crypto_module() -> NativeModule {
    let mut functions: HashMap<&'static str, NativeFunctionInfo> = HashMap::new();

    functions.insert(
        "ENCRYPT",
        NativeFunctionInfo {
            func: crypto::encrypt,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "DECRYPT",
        NativeFunctionInfo {
            func: crypto::decrypt,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "HASH",
        NativeFunctionInfo {
            func: crypto::hash,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "BASE64_ENCODE",
        NativeFunctionInfo {
            func: crypto::base64_encode,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "BASE64_DECODE",
        NativeFunctionInfo {
            func: crypto::base64_decode,
            arity: Arity::Exact(1),
        },
    );
    functions.insert(
        "HMAC_GENERATE",
        NativeFunctionInfo {
            func: crypto::hmac_generate,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "HMAC_VERIFY",
        NativeFunctionInfo {
            func: crypto::hmac_verify,
            arity: Arity::Exact(3),
        },
    );

    functions.insert(
        "AES_ENCRYPT",
        NativeFunctionInfo {
            func: crypto::aes_encrypt,
            arity: Arity::Exact(3),
        },
    );
    functions.insert(
        "AES_DECRYPT",
        NativeFunctionInfo {
            func: crypto::aes_decrypt,
            arity: Arity::Exact(3),
        },
    );

    functions.insert(
        "RSA_GENERATE_KEY",
        NativeFunctionInfo {
            func: crypto::rsa_generate_key,
            arity: Arity::Exact(0),
        },
    );
    functions.insert(
        "RSA_ENCRYPT",
        NativeFunctionInfo {
            func: crypto::rsa_encrypt,
            arity: Arity::Exact(2),
        },
    );
    functions.insert(
        "RSA_DECRYPT",
        NativeFunctionInfo {
            func: crypto::rsa_decrypt,
            arity: Arity::Exact(2),
        },
    );

    NativeModule {
        functions,
        constants: HashMap::new(),
    }
}
