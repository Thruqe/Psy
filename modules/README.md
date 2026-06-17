# Pseudocode Interpreter Modules

This directory contains built-in modules that can be imported into pseudocode programs.

## Available Modules

### `_MATH`

Mathematical functions and constants including trigonometric functions, rounding, and statistical operations.

**Import:**

```pseudocode
IMPORT _MATH
```

**Functions:** `SIN`, `COS`, `TAN`, `SQRT`, `POW`, `ABS`, `ROUND`, `FLOOR`, `CEIL`, `MEAN`, `MEDIAN`, `MODE`, `VARIANCE`, `STDDEV`, `MIN`, `MAX`, `SUM`, `PRODUCT`

**Constants:** `PI`, `E`

---

### `_OS` (Planned)

Operating system interaction functions for file operations, environment variables, and system calls.

---

### `_ASYNC` (Planned)

Asynchronous programming support for concurrent and parallel operations.

---

### `_CRYPTO` (Planned)

Cryptographic functions for hashing, encryption, and secure random number generation.

---

### `_NETWORK` (Planned)

Networking functions for HTTP requests, WebSocket connections, and network communication.

---

### `_FS` (Planned)

File system operations for reading, writing, and managing files and directories.

---

### `_TIME` (Planned)

Time and date functions for working with timestamps, durations, and calendars.

---

### `_UTIL` (Planned)

Utility functions for common programming tasks and helpers.

---

## Module Structure

Each module follows this structure:

```
modules/
└── [module_name]/
    ├── Cargo.toml
    ├── README.md
    └── src/
        └── lib.rs
```

## Importing Modules

### Import Entire Module

```pseudocode
IMPORT _MODULE
```

### Import Specific Functions

```pseudocode
IMPORT _MODULE[FUN1, FUN2, FUN3]
```

### Import Multiple Modules

```pseudocode
IMPORT _MODULE1, _MODULE2
```

### Import with Aliases (Future Feature)

```pseudocode
IMPORT _MODULE1 AS MODULE_SOMETHING_ELSE
```

## Creating a New Module

To create a new module:

1. Create a directory in `modules/`
2. Add a `Cargo.toml` file
3. Create a `src/lib.rs` file
4. Implement functions with signature:
   ```rust
   pub fn function_name(args: &[Value]) -> Result<Value, String>
   ```
5. Register the module in [here](../core/src/interpreter/native.rs)

## Module Requirements

- Each module must export functions that accept `&[Value]` and return `Result<Value, String>`
- Functions should handle input validation and provide clear error messages
- Modules should be documented with usage examples
- Constants should be provided where appropriate

## Notes

- All modules are implemented in Rust for performance and safety
- Modules are loaded dynamically when imported
- Functions are evaluated with proper error handling
- Constants are available as read-only values in the environment

For more information, see the [main README](../README.md) or individual module documentation.
