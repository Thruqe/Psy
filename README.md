<p align="center">
  <strong>The Psy Programming Language</strong><br>
  <em>(Parsable Syntax for You)</em>
</p>

<p align="center">
  <em>A super simple and fast interpreter for executing pseudocode written in a custom educational language.</em>
</p>

<p align="center">
  This interpreter lexes, parses, and evaluates pseudocode programs, making it ideal for learning programming concepts and algorithm visualization.
</p>

<p align="center">
  <a href="./examples/">Syntax Examples</a> · 
  <a href="./core/">Core System</a> · 
  <a href="./checker/">Syntax Checker</a> · 
  <a href="./lsp/">Language Server</a> · 
  <a href="./modules/">Native Modules</a>
</p>

---

## Features

### Core Interpreter Pipeline
- **Lexer**: Scans source code and converts it into a stream of tokens
- **Parser**: Builds an Abstract Syntax Tree (AST) from the token stream
- **Evaluator**: Walks the AST and executes the program logic
- **Error Reporting**: Syntax and runtime errors with line numbers and helpful messages

### Language Constructs
- **Program Structure**: Programs are wrapped in `START` and `END` blocks
- **Variables**: Can be declared with `DECLARE` or used directly with assignment (`=`)
- **Constants**: Declared using `CONST` (value cannot be reassigned)
- **Data Types**: Integers, floats, strings, and booleans
- **Arrays**: Declared with size (e.g., `DECLARE scores[5]`) and accessed via index (e.g., `scores[0]`)
- **Assignment**: Variables, constants, and array elements assigned using `=`
- **Functions**: User-defined functions declared with `FUNCTION` keyword
- **Control Flow**: `FOR` loops with `TO` syntax (e.g., `FOR i = 0 TO 4`) and `ENDFOR`
- **Input/Output**: `OUTPUT` for console output
- **Arithmetic**: Standard mathematical operations (`+`, `-`, `*`, `/`, `%`)
- **Comparisons**: Equality and relational operators (`==`, `!=`, `<`, `>`, `<=`, `>=`)
- **Logical Operators**: Boolean logic (`AND`, `OR`, `NOT`)

### Debugging
- Built-in debug mode enabled with `--debug` flag
- Step-by-step execution tracing
- Variable state inspection during runtime

### Educational Focus
- Designed specifically for teaching fundamental programming concepts
- Syntax mirrors common pseudocode conventions used in textbooks
- Clear, beginner-friendly error messages to aid learning

### Modular Architecture
- **Core System**: The main interpreter engine
- **Syntax Checker**: Validates code structure before execution
- **Language Server**: Provides IDE integration (autocomplete, diagnostics)
- **Native Modules**: Extend functionality with external modules

---

## Contributing

Contributions are welcome! Feel free to open issues, submit pull requests, or suggest improvements.

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.