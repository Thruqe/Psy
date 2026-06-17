# Pseudocode Interpreter (psc)

A super simple and fast interpreter for executing pseudocode written in a custom educational language. This interpreter lexes, parses, and evaluates pseudocode programs, making it ideal for learning programming concepts and algorithm visualization.

[**_Core System_**](./core/)

[**_Syntax Checker_**](./checker/)

[**_Language Server_**](./lsp/)

[**_Native Modules_**](./modules/)

## Features

- **Complete Interpreter Pipeline**: Lexer → Parser → Evaluator
- **Rich Language Features**:
  - Variables and data types (integers, floats, strings, booleans)
  - Arrays and array operations
  - Control flow (if/else, loops)
  - Functions and procedures
  - Input/Output operations
  - Mathematical operations
  - Logical operators and comparisons
- **Debugging Support**: Built-in debug mode for step-by-step execution
- **Clear Error Messages**: Informative syntax and runtime error reporting
- **Educational**: Designed for teaching programming concepts

## Language Syntax Examples

### Variables

```psc
x ← 10
name ← "Alice"
is_valid ← TRUE
```

### Arrays

```psc
arr ← [1, 2, 3, 4, 5]
arr[2] ← 10
```

### Input/Output

```psc
OUTPUT "Enter your name: "
INPUT name
OUTPUT "Hello, " + name
```

### Conditionals

```psc
IF x > 10 THEN
OUTPUT "x is greater than 10"
ELSE
OUTPUT "x is less than or equal to 10"
ENDIF
```

### Loops

```psc
FOR i ← 1 TO 5 DO
OUTPUT i
ENDFOR

WHILE x < 10 DO
x ← x + 1
ENDWHILE
```

### Functions

```psc
FUNCTION factorial(n)
IF n ≤ 1 THEN
RETURN 1
ELSE
RETURN n * factorial(n - 1)
ENDIF
ENDFUNCTION
```

Here's the simplified version:

## Contributing

Contributions are welcome! Feel free to open issues, submit pull requests, or suggest improvements.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
