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
    pub description: &'static str,
    pub return_type: &'static str,
    pub parameters: &'static [(&'static str, &'static str)], // (name, type)
}

pub struct NativeConstantInfo {
    pub value: Value,
    pub description: &'static str,
    pub constant_type: &'static str,
}

pub struct NativeModule {
    pub name: &'static str,
    pub description: &'static str,
    pub functions: HashMap<&'static str, NativeFunctionInfo>,
    pub constants: HashMap<&'static str, NativeConstantInfo>,
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

    pub fn get_function_info(&self, name: &str) -> Option<&NativeFunctionInfo> {
        self.functions.get(name)
    }

    pub fn get_constant_info(&self, name: &str) -> Option<&NativeConstantInfo> {
        self.constants.get(name)
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn has_constant(&self, name: &str) -> bool {
        self.constants.contains_key(name)
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
        "_NETWORK" => Some(network_module()),
        "_JSON" => Some(json_module()),
        "_ASYNC" => Some(async_module()),
        _ => None,
    }
}

/// Returns all available native module names
pub fn module_names() -> Vec<&'static str> {
    vec![
        "_MATH", "_FS", "_TIME", "_CRYPTO", "_NETWORK", "_JSON", "_ASYNC",
    ]
}

fn math_module() -> NativeModule {
    let mut functions: HashMap<&'static str, NativeFunctionInfo> = HashMap::new();

    functions.insert(
        "SIN",
        NativeFunctionInfo {
            func: math::sin,
            arity: Arity::Exact(1),
            description: "Calculates the sine of an angle in radians",
            return_type: "Number",
            parameters: &[("angle", "Number")],
        },
    );
    functions.insert(
        "COS",
        NativeFunctionInfo {
            func: math::cos,
            arity: Arity::Exact(1),
            description: "Calculates the cosine of an angle in radians",
            return_type: "Number",
            parameters: &[("angle", "Number")],
        },
    );
    functions.insert(
        "TAN",
        NativeFunctionInfo {
            func: math::tan,
            arity: Arity::Exact(1),
            description: "Calculates the tangent of an angle in radians",
            return_type: "Number",
            parameters: &[("angle", "Number")],
        },
    );
    functions.insert(
        "SQRT",
        NativeFunctionInfo {
            func: math::sqrt,
            arity: Arity::Exact(1),
            description: "Calculates the square root of a number",
            return_type: "Number",
            parameters: &[("value", "Number")],
        },
    );
    functions.insert(
        "POW",
        NativeFunctionInfo {
            func: math::pow,
            arity: Arity::Exact(2),
            description: "Raises base to the exponent power",
            return_type: "Number",
            parameters: &[("base", "Number"), ("exponent", "Number")],
        },
    );
    functions.insert(
        "ABS",
        NativeFunctionInfo {
            func: math::abs,
            arity: Arity::Exact(1),
            description: "Returns the absolute value of a number",
            return_type: "Number",
            parameters: &[("value", "Number")],
        },
    );
    functions.insert(
        "ROUND",
        NativeFunctionInfo {
            func: math::round,
            arity: Arity::Exact(1),
            description: "Rounds a number to the nearest integer",
            return_type: "Number",
            parameters: &[("value", "Number")],
        },
    );
    functions.insert(
        "FLOOR",
        NativeFunctionInfo {
            func: math::floor,
            arity: Arity::Exact(1),
            description: "Rounds a number down to the nearest integer",
            return_type: "Number",
            parameters: &[("value", "Number")],
        },
    );
    functions.insert(
        "CEIL",
        NativeFunctionInfo {
            func: math::ceil,
            arity: Arity::Exact(1),
            description: "Rounds a number up to the nearest integer",
            return_type: "Number",
            parameters: &[("value", "Number")],
        },
    );
    functions.insert(
        "MEAN",
        NativeFunctionInfo {
            func: math::mean,
            arity: Arity::AtLeast(1),
            description: "Calculates the arithmetic mean of a collection of numbers",
            return_type: "Number",
            parameters: &[("numbers", "Number...")],
        },
    );
    functions.insert(
        "MEDIAN",
        NativeFunctionInfo {
            func: math::median,
            arity: Arity::AtLeast(1),
            description: "Finds the median value in a collection of numbers",
            return_type: "Number",
            parameters: &[("numbers", "Number...")],
        },
    );
    functions.insert(
        "MODE",
        NativeFunctionInfo {
            func: math::mode,
            arity: Arity::AtLeast(1),
            description: "Finds the most frequently occurring value in a collection",
            return_type: "Number",
            parameters: &[("numbers", "Number...")],
        },
    );
    functions.insert(
        "VARIANCE",
        NativeFunctionInfo {
            func: math::variance,
            arity: Arity::AtLeast(1),
            description: "Calculates the variance of a collection of numbers",
            return_type: "Number",
            parameters: &[("numbers", "Number...")],
        },
    );
    functions.insert(
        "STDDEV",
        NativeFunctionInfo {
            func: math::stddev,
            arity: Arity::AtLeast(1),
            description: "Calculates the standard deviation of a collection of numbers",
            return_type: "Number",
            parameters: &[("numbers", "Number...")],
        },
    );
    functions.insert(
        "MIN",
        NativeFunctionInfo {
            func: math::min,
            arity: Arity::AtLeast(1),
            description: "Returns the smallest value in a collection",
            return_type: "Number",
            parameters: &[("numbers", "Number...")],
        },
    );
    functions.insert(
        "MAX",
        NativeFunctionInfo {
            func: math::max,
            arity: Arity::AtLeast(1),
            description: "Returns the largest value in a collection",
            return_type: "Number",
            parameters: &[("numbers", "Number...")],
        },
    );
    functions.insert(
        "SUM",
        NativeFunctionInfo {
            func: math::sum,
            arity: Arity::AtLeast(1),
            description: "Calculates the sum of a collection of numbers",
            return_type: "Number",
            parameters: &[("numbers", "Number...")],
        },
    );
    functions.insert(
        "PRODUCT",
        NativeFunctionInfo {
            func: math::product,
            arity: Arity::AtLeast(1),
            description: "Calculates the product of a collection of numbers",
            return_type: "Number",
            parameters: &[("numbers", "Number...")],
        },
    );
    functions.insert(
        "GCD",
        NativeFunctionInfo {
            func: math::gcd,
            arity: Arity::Exact(2),
            description: "Calculates the greatest common divisor of two numbers",
            return_type: "Number",
            parameters: &[("a", "Number"), ("b", "Number")],
        },
    );
    functions.insert(
        "LCM",
        NativeFunctionInfo {
            func: math::lcm,
            arity: Arity::Exact(2),
            description: "Calculates the least common multiple of two numbers",
            return_type: "Number",
            parameters: &[("a", "Number"), ("b", "Number")],
        },
    );
    functions.insert(
        "IS_PRIME",
        NativeFunctionInfo {
            func: math::is_prime,
            arity: Arity::Exact(1),
            description: "Tests if a number is prime",
            return_type: "Boolean",
            parameters: &[("n", "Number")],
        },
    );
    functions.insert(
        "MATRIX_ADD",
        NativeFunctionInfo {
            func: math::matrix_add,
            arity: Arity::AtLeast(2),
            description: "Adds two or more matrices element-wise",
            return_type: "Array",
            parameters: &[("matrices", "Array...")],
        },
    );
    functions.insert(
        "MATRIX_MULTIPLY",
        NativeFunctionInfo {
            func: math::matrix_multiply,
            arity: Arity::AtLeast(2),
            description: "Multiplies two or more matrices",
            return_type: "Array",
            parameters: &[("matrices", "Array...")],
        },
    );
    functions.insert(
        "MATRIX_TRANSPOSE",
        NativeFunctionInfo {
            func: math::matrix_transpose,
            arity: Arity::AtLeast(1),
            description: "Transposes a matrix (flips rows and columns)",
            return_type: "Array",
            parameters: &[("matrix", "Array")],
        },
    );
    functions.insert(
        "MATRIX_DETERMINANT",
        NativeFunctionInfo {
            func: math::matrix_determinant,
            arity: Arity::AtLeast(1),
            description: "Calculates the determinant of a square matrix",
            return_type: "Number",
            parameters: &[("matrix", "Array")],
        },
    );
    functions.insert(
        "MATRIX_INVERSE",
        NativeFunctionInfo {
            func: math::matrix_inverse,
            arity: Arity::AtLeast(1),
            description: "Calculates the inverse of a square matrix",
            return_type: "Array",
            parameters: &[("matrix", "Array")],
        },
    );
    functions.insert(
        "DOT",
        NativeFunctionInfo {
            func: math::dot,
            arity: Arity::Exact(2),
            description: "Calculates the dot product of two vectors",
            return_type: "Number",
            parameters: &[("vector_a", "Array"), ("vector_b", "Array")],
        },
    );
    functions.insert(
        "CROSS",
        NativeFunctionInfo {
            func: math::cross,
            arity: Arity::Exact(2),
            description: "Calculates the cross product of two 3D vectors",
            return_type: "Array",
            parameters: &[("vector_a", "Array"), ("vector_b", "Array")],
        },
    );
    functions.insert(
        "ASIN",
        NativeFunctionInfo {
            func: math::asin,
            arity: Arity::Exact(1),
            description: "Calculates the inverse sine (arcsine) in radians",
            return_type: "Number",
            parameters: &[("value", "Number")],
        },
    );
    functions.insert(
        "ACOS",
        NativeFunctionInfo {
            func: math::acos,
            arity: Arity::Exact(1),
            description: "Calculates the inverse cosine (arccosine) in radians",
            return_type: "Number",
            parameters: &[("value", "Number")],
        },
    );
    functions.insert(
        "ATAN",
        NativeFunctionInfo {
            func: math::atan,
            arity: Arity::Exact(1),
            description: "Calculates the inverse tangent (arctangent) in radians",
            return_type: "Number",
            parameters: &[("value", "Number")],
        },
    );
    functions.insert(
        "LOG",
        NativeFunctionInfo {
            func: math::log,
            arity: Arity::Exact(1),
            description: "Calculates the natural logarithm of a number",
            return_type: "Number",
            parameters: &[("value", "Number")],
        },
    );
    functions.insert(
        "LOG10",
        NativeFunctionInfo {
            func: math::log10,
            arity: Arity::Exact(1),
            description: "Calculates the base-10 logarithm of a number",
            return_type: "Number",
            parameters: &[("value", "Number")],
        },
    );
    functions.insert(
        "EXP",
        NativeFunctionInfo {
            func: math::exp,
            arity: Arity::Exact(1),
            description: "Calculates e raised to the given power",
            return_type: "Number",
            parameters: &[("exponent", "Number")],
        },
    );
    functions.insert(
        "FACTORIAL",
        NativeFunctionInfo {
            func: math::factorial,
            arity: Arity::Exact(1),
            description: "Calculates the factorial of a non-negative integer",
            return_type: "Number",
            parameters: &[("n", "Number")],
        },
    );

    let mut constants: HashMap<&'static str, NativeConstantInfo> = HashMap::new();
    constants.insert(
        "PI",
        NativeConstantInfo {
            value: math::pi(),
            description: "The mathematical constant π (pi)",
            constant_type: "Number",
        },
    );
    constants.insert(
        "E",
        NativeConstantInfo {
            value: math::e(),
            description: "The mathematical constant e (Euler's number)",
            constant_type: "Number",
        },
    );

    NativeModule {
        name: "_MATH",
        description: "Mathematical functions and constants for advanced calculations",
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
            description: "Reads the contents of a file as a string",
            return_type: "String",
            parameters: &[("path", "String")],
        },
    );
    functions.insert(
        "WRITEFILE",
        NativeFunctionInfo {
            func: fs::write_file,
            arity: Arity::Exact(2),
            description: "Writes string content to a file",
            return_type: "Void",
            parameters: &[("path", "String"), ("content", "String")],
        },
    );
    functions.insert(
        "EXISTS",
        NativeFunctionInfo {
            func: fs::exists,
            arity: Arity::Exact(1),
            description: "Checks if a file or directory exists at the given path",
            return_type: "Boolean",
            parameters: &[("path", "String")],
        },
    );
    functions.insert(
        "ISFILE",
        NativeFunctionInfo {
            func: fs::is_file,
            arity: Arity::Exact(1),
            description: "Checks if the path points to a file",
            return_type: "Boolean",
            parameters: &[("path", "String")],
        },
    );
    functions.insert(
        "ISDIR",
        NativeFunctionInfo {
            func: fs::is_dir,
            arity: Arity::Exact(1),
            description: "Checks if the path points to a directory",
            return_type: "Boolean",
            parameters: &[("path", "String")],
        },
    );
    functions.insert(
        "DELETE",
        NativeFunctionInfo {
            func: fs::delete,
            arity: Arity::Exact(1),
            description: "Deletes a file or empty directory",
            return_type: "Boolean",
            parameters: &[("path", "String")],
        },
    );
    functions.insert(
        "LISTDIR",
        NativeFunctionInfo {
            func: fs::list_dir,
            arity: Arity::Exact(1),
            description: "Lists the contents of a directory as an array",
            return_type: "Array",
            parameters: &[("path", "String")],
        },
    );

    NativeModule {
        name: "_FS",
        description: "File system operations for reading, writing, and managing files",
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
            description: "Returns the current system time as a timestamp",
            return_type: "Number",
            parameters: &[],
        },
    );
    functions.insert(
        "NOWMS",
        NativeFunctionInfo {
            func: time::now_ms,
            arity: Arity::Exact(0),
            description: "Returns the current system time in milliseconds",
            return_type: "Number",
            parameters: &[],
        },
    );
    functions.insert(
        "SLEEP",
        NativeFunctionInfo {
            func: time::sleep,
            arity: Arity::Exact(1),
            description: "Pauses execution for a specified number of seconds",
            return_type: "Void",
            parameters: &[("seconds", "Number")],
        },
    );
    functions.insert(
        "SLEEPMS",
        NativeFunctionInfo {
            func: time::sleep_ms,
            arity: Arity::Exact(1),
            description: "Pauses execution for a specified number of milliseconds",
            return_type: "Void",
            parameters: &[("milliseconds", "Number")],
        },
    );
    functions.insert(
        "FORMATTIME",
        NativeFunctionInfo {
            func: time::format_time,
            arity: Arity::Exact(1),
            description: "Formats a timestamp into a human-readable date/time string",
            return_type: "String",
            parameters: &[("timestamp", "Number")],
        },
    );

    NativeModule {
        name: "_TIME",
        description: "Time utilities for getting system time and controlling execution flow",
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
            description: "Encrypts a string using a provided key",
            return_type: "String",
            parameters: &[("text", "String"), ("key", "String")],
        },
    );
    functions.insert(
        "DECRYPT",
        NativeFunctionInfo {
            func: crypto::decrypt,
            arity: Arity::Exact(2),
            description: "Decrypts a string using a provided key",
            return_type: "String",
            parameters: &[("encrypted_text", "String"), ("key", "String")],
        },
    );
    functions.insert(
        "HASH",
        NativeFunctionInfo {
            func: crypto::hash,
            arity: Arity::Exact(1),
            description: "Generates a hash of the input string",
            return_type: "String",
            parameters: &[("text", "String")],
        },
    );
    functions.insert(
        "BASE64_ENCODE",
        NativeFunctionInfo {
            func: crypto::base64_encode,
            arity: Arity::Exact(1),
            description: "Encodes a string to Base64 format",
            return_type: "String",
            parameters: &[("text", "String")],
        },
    );
    functions.insert(
        "BASE64_DECODE",
        NativeFunctionInfo {
            func: crypto::base64_decode,
            arity: Arity::Exact(1),
            description: "Decodes a Base64 string back to its original form",
            return_type: "String",
            parameters: &[("encoded_text", "String")],
        },
    );
    functions.insert(
        "HMAC_GENERATE",
        NativeFunctionInfo {
            func: crypto::hmac_generate,
            arity: Arity::Exact(2),
            description: "Generates an HMAC signature for a message with a secret key",
            return_type: "String",
            parameters: &[("message", "String"), ("secret", "String")],
        },
    );
    functions.insert(
        "HMAC_VERIFY",
        NativeFunctionInfo {
            func: crypto::hmac_verify,
            arity: Arity::Exact(3),
            description: "Verifies an HMAC signature against a message",
            return_type: "Boolean",
            parameters: &[
                ("message", "String"),
                ("secret", "String"),
                ("signature", "String"),
            ],
        },
    );
    functions.insert(
        "AES_ENCRYPT",
        NativeFunctionInfo {
            func: crypto::aes_encrypt,
            arity: Arity::Exact(3),
            description: "Encrypts data using AES encryption",
            return_type: "String",
            parameters: &[("data", "String"), ("key", "String"), ("iv", "String")],
        },
    );
    functions.insert(
        "AES_DECRYPT",
        NativeFunctionInfo {
            func: crypto::aes_decrypt,
            arity: Arity::Exact(3),
            description: "Decrypts AES-encrypted data",
            return_type: "String",
            parameters: &[
                ("encrypted_data", "String"),
                ("key", "String"),
                ("iv", "String"),
            ],
        },
    );
    functions.insert(
        "RSA_GENERATE_KEY",
        NativeFunctionInfo {
            func: crypto::rsa_generate_key,
            arity: Arity::Exact(0),
            description: "Generates an RSA key pair",
            return_type: "String",
            parameters: &[],
        },
    );
    functions.insert(
        "RSA_ENCRYPT",
        NativeFunctionInfo {
            func: crypto::rsa_encrypt,
            arity: Arity::Exact(2),
            description: "Encrypts data using RSA public key encryption",
            return_type: "String",
            parameters: &[("data", "String"), ("public_key", "String")],
        },
    );
    functions.insert(
        "RSA_DECRYPT",
        NativeFunctionInfo {
            func: crypto::rsa_decrypt,
            arity: Arity::Exact(2),
            description: "Decrypts RSA-encrypted data using a private key",
            return_type: "String",
            parameters: &[("encrypted_data", "String"), ("private_key", "String")],
        },
    );

    NativeModule {
        name: "_CRYPTO",
        description: "Cryptographic operations including encryption, hashing, and encoding",
        functions,
        constants: HashMap::new(),
    }
}

fn network_module() -> NativeModule {
    let mut functions: HashMap<&'static str, NativeFunctionInfo> = HashMap::new();

    functions.insert(
        "HTTP_GET",
        NativeFunctionInfo {
            func: network::http_get,
            arity: Arity::Exact(1),
            description: "Performs an HTTP GET request to the specified URL",
            return_type: "Array",
            parameters: &[("url", "String")],
        },
    );

    functions.insert(
        "HTTP_POST",
        NativeFunctionInfo {
            func: network::http_post,
            arity: Arity::Exact(2),
            description: "Performs an HTTP POST request with a body to the specified URL",
            return_type: "Array",
            parameters: &[("url", "String"), ("body", "String")],
        },
    );

    functions.insert(
        "HTTP_PUT",
        NativeFunctionInfo {
            func: network::http_put,
            arity: Arity::Exact(2),
            description: "Performs an HTTP PUT request with a body to the specified URL",
            return_type: "Array",
            parameters: &[("url", "String"), ("body", "String")],
        },
    );

    functions.insert(
        "HTTP_DELETE",
        NativeFunctionInfo {
            func: network::http_delete,
            arity: Arity::Exact(1),
            description: "Performs an HTTP DELETE request to the specified URL",
            return_type: "Array",
            parameters: &[("url", "String")],
        },
    );

    functions.insert(
        "HTTP_HEAD",
        NativeFunctionInfo {
            func: network::http_head,
            arity: Arity::Exact(1),
            description: "Performs an HTTP HEAD request to retrieve headers only",
            return_type: "Array",
            parameters: &[("url", "String")],
        },
    );

    functions.insert(
        "URL_ENCODE",
        NativeFunctionInfo {
            func: network::url_encode,
            arity: Arity::Exact(1),
            description: "Encodes a string for safe use in URLs",
            return_type: "String",
            parameters: &[("text", "String")],
        },
    );

    functions.insert(
        "URL_DECODE",
        NativeFunctionInfo {
            func: network::url_decode,
            arity: Arity::Exact(1),
            description: "Decodes a URL-encoded string",
            return_type: "String",
            parameters: &[("text", "String")],
        },
    );

    functions.insert(
        "PARSE_JSON",
        NativeFunctionInfo {
            func: network::parse_json,
            arity: Arity::Exact(1),
            description: "Parses a JSON string into a Psy array",
            return_type: "Array",
            parameters: &[("json", "String")],
        },
    );

    functions.insert(
        "SERVER_CREATE",
        NativeFunctionInfo {
            func: network::server_create,
            arity: Arity::Exact(1),
            description: "Creates a new HTTP server on the specified port",
            return_type: "String",
            parameters: &[("port", "Number")],
        },
    );

    functions.insert(
        "SERVER_LISTEN",
        NativeFunctionInfo {
            func: network::server_listen,
            arity: Arity::Exact(1),
            description: "Starts listening on the server",
            return_type: "Boolean",
            parameters: &[("server_id", "String")],
        },
    );

    functions.insert(
        "SERVER_ROUTE",
        NativeFunctionInfo {
            func: network::server_route,
            arity: Arity::Exact(3),
            description: "Adds a route to the server (method, path)",
            return_type: "Boolean",
            parameters: &[
                ("server_id", "String"),
                ("method", "String"),
                ("path", "String"),
            ],
        },
    );

    functions.insert(
        "SERVER_ACCEPT",
        NativeFunctionInfo {
            func: network::server_accept,
            arity: Arity::Exact(1),
            description: "Accepts one incoming connection (non-blocking)",
            return_type: "String",
            parameters: &[("server_id", "String")],
        },
    );

    functions.insert(
        "SERVER_ACCEPT_BLOCKING",
        NativeFunctionInfo {
            func: network::server_accept_blocking,
            arity: Arity::Exact(1),
            description: "Accepts one connection and returns the raw request",
            return_type: "Array",
            parameters: &[("server_id", "String")],
        },
    );

    functions.insert(
        "SERVER_RESPOND",
        NativeFunctionInfo {
            func: network::server_respond,
            arity: Arity::AtLeast(2),
            description: "Sends an HTTP response (request_id, body, status_code?)",
            return_type: "Boolean",
            parameters: &[
                ("request_id", "String"),
                ("body", "String"),
                ("status", "Number"),
            ],
        },
    );

    functions.insert(
        "SERVER_STOP",
        NativeFunctionInfo {
            func: network::server_stop,
            arity: Arity::Exact(1),
            description: "Stops and removes a server",
            return_type: "Boolean",
            parameters: &[("server_id", "String")],
        },
    );

    functions.insert(
        "SERVER_LIST_ALL",
        NativeFunctionInfo {
            func: network::server_list,
            arity: Arity::Exact(0),
            description: "Lists all active servers",
            return_type: "Array",
            parameters: &[],
        },
    );

    functions.insert(
        "WS_CONNECT",
        NativeFunctionInfo {
            func: network::websocket_connect,
            arity: Arity::Exact(1),
            description: "Opens a WebSocket connection to the specified URL",
            return_type: "String",
            parameters: &[("url", "String")],
        },
    );

    functions.insert(
        "WS_SEND",
        NativeFunctionInfo {
            func: network::websocket_send,
            arity: Arity::Exact(2),
            description: "Sends a message over a WebSocket connection",
            return_type: "Boolean",
            parameters: &[("connection_id", "String"), ("message", "String")],
        },
    );

    functions.insert(
        "WS_RECEIVE",
        NativeFunctionInfo {
            func: network::websocket_receive,
            arity: Arity::Exact(1),
            description: "Receives the next message from a WebSocket connection",
            return_type: "String",
            parameters: &[("connection_id", "String")],
        },
    );

    functions.insert(
        "WS_CLOSE",
        NativeFunctionInfo {
            func: network::websocket_close,
            arity: Arity::Exact(1),
            description: "Closes a WebSocket connection",
            return_type: "Boolean",
            parameters: &[("connection_id", "String")],
        },
    );

    NativeModule {
        name: "_NETWORK",
        description: "Network operations for HTTP requests, URL encoding, and JSON parsing",
        functions,
        constants: HashMap::new(),
    }
}

fn json_module() -> NativeModule {
    let mut functions: HashMap<&'static str, NativeFunctionInfo> = HashMap::new();

    functions.insert(
        "JSON_PARSE",
        NativeFunctionInfo {
            func: json::json_parse,
            arity: Arity::Exact(1),
            description: "Parses a JSON string into a Psy value",
            return_type: "Array",
            parameters: &[("json_string", "String")],
        },
    );

    functions.insert(
        "JSON_STRINGIFY",
        NativeFunctionInfo {
            func: json::json_stringify,
            arity: Arity::Exact(1),
            description: "Converts a Psy value to a JSON string",
            return_type: "String",
            parameters: &[("value", "Any")],
        },
    );

    functions.insert(
        "JSON_GET",
        NativeFunctionInfo {
            func: json::json_get,
            arity: Arity::Exact(2),
            description: "Gets a value from a JSON string by key or index",
            return_type: "Any",
            parameters: &[("json_string", "String"), ("key", "String/Number")],
        },
    );

    functions.insert(
        "JSON_SET",
        NativeFunctionInfo {
            func: json::json_set,
            arity: Arity::Exact(3),
            description: "Sets a value in a JSON string by key or index",
            return_type: "String",
            parameters: &[
                ("json_string", "String"),
                ("key", "String/Number"),
                ("value", "Any"),
            ],
        },
    );

    functions.insert(
        "JSON_KEYS",
        NativeFunctionInfo {
            func: json::json_keys,
            arity: Arity::Exact(1),
            description: "Gets all keys from a JSON object string",
            return_type: "Array",
            parameters: &[("json_string", "String")],
        },
    );

    NativeModule {
        name: "_JSON",
        description: "JSON parsing, stringification, and manipulation functions",
        functions,
        constants: HashMap::new(),
    }
}

fn async_module() -> NativeModule {
    let mut functions: HashMap<&'static str, NativeFunctionInfo> = HashMap::new();

    functions.insert(
        "ASYNC_RUN",
        NativeFunctionInfo {
            func: async_psy::async_run,
            arity: Arity::Exact(1),
            description: "Creates an async task",
            return_type: "String",
            parameters: &[("task_name", "String")],
        },
    );

    functions.insert(
        "ASYNC_SPAWN",
        NativeFunctionInfo {
            func: async_psy::async_spawn,
            arity: Arity::Exact(2),
            description: "Spawns an async task in a separate thread",
            return_type: "Boolean",
            parameters: &[("task_id", "String"), ("function", "String")],
        },
    );

    functions.insert(
        "ASYNC_AWAIT",
        NativeFunctionInfo {
            func: async_psy::async_await,
            arity: Arity::Exact(1),
            description: "Awaits completion of an async task",
            return_type: "Any",
            parameters: &[("task_id", "String")],
        },
    );

    functions.insert(
        "ASYNC_SLEEP",
        NativeFunctionInfo {
            func: async_psy::async_sleep,
            arity: Arity::Exact(1),
            description: "Sleeps asynchronously for specified milliseconds",
            return_type: "Void",
            parameters: &[("milliseconds", "Number")],
        },
    );

    functions.insert(
        "ASYNC_PARALLEL",
        NativeFunctionInfo {
            func: async_psy::async_parallel,
            arity: Arity::Exact(1),
            description: "Runs multiple tasks in parallel",
            return_type: "Array",
            parameters: &[("tasks", "Array")],
        },
    );

    functions.insert(
        "ASYNC_STATUS",
        NativeFunctionInfo {
            func: async_psy::async_status,
            arity: Arity::Exact(1),
            description: "Gets the status of an async task",
            return_type: "String",
            parameters: &[("task_id", "String")],
        },
    );

    functions.insert(
        "ASYNC_CANCEL",
        NativeFunctionInfo {
            func: async_psy::async_cancel,
            arity: Arity::Exact(1),
            description: "Cancels an async task",
            return_type: "Boolean",
            parameters: &[("task_id", "String")],
        },
    );

    functions.insert(
        "ASYNC_AWAIT_ALL",
        NativeFunctionInfo {
            func: async_psy::async_await_all,
            arity: Arity::Exact(1),
            description: "Awaits completion of multiple async tasks",
            return_type: "Array",
            parameters: &[("task_ids", "Array")],
        },
    );

    NativeModule {
        name: "_ASYNC",
        description: "Asynchronous programming support with tasks, parallel execution, and channels",
        functions,
        constants: HashMap::new(),
    }
}
