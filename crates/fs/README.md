# Psy FS Module

A built-in file system module for the Psy Interpreter providing common file and directory operations.

## Overview

The `_FS` module provides essential file system operations for Psy programs, including reading/writing files, checking file existence, listing directories, and file/directory deletion. All paths are relative to the current working directory where the Psy interpreter is executed.

## Importing the Module

To use the fs module in your psy program:

```psy
IMPORT _FS
```

To import only specific functions:

```psy
IMPORT _FS[READFILE, WRITEFILE, EXISTS]
```

## Available Functions

### File Operations

| Function                   | Description            | Example                          | Result        |
| -------------------------- | ---------------------- | -------------------------------- | ------------- |
| `READFILE(path)`           | Read entire file       | `READFILE("input.txt")`          | File contents |
| `WRITEFILE(path, content)` | Write content to file  | `WRITEFILE("out.txt", "Hello")`  | `undefined`   |

### File System Information

| Function      | Description                   | Example           | Result    |
| ------------- | ----------------------------- | ----------------- | --------- |
| `EXISTS(path)` | Check if path exists          | `EXISTS("file.txt")` | `TRUE`/`FALSE` |
| `ISFILE(path)`  | Check if path is a file      | `ISFILE("test.psy")` | `TRUE`/`FALSE` |
| `ISDIR(path)`   | Check if path is a directory | `ISDIR("modules")`   | `TRUE`/`FALSE` |

### Directory Operations

| Function       | Description                   | Example          | Result                     |
| -------------- | ----------------------------- | ---------------- | -------------------------- |
| `LISTDIR(path)` | List directory contents       | `LISTDIR(".")`   | Array of file/folder names |

### File Management

| Function     | Description                     | Example              | Result      |
| ------------ | ------------------------------- | -------------------- | ----------- |
| `DELETE(path)` | Delete a file or directory      | `DELETE("old.txt")`  | `undefined` |

## Function Signatures

### File I/O

- `READFILE(string path) -> string` - Reads and returns the entire contents of a file
- `WRITEFILE(string path, string content) -> undefined` - Writes content to a file, creating it if it doesn't exist

### File System Queries

- `EXISTS(string path) -> boolean` - Returns TRUE if the path exists, FALSE otherwise
- `ISFILE(string path) -> boolean` - Returns TRUE if the path is a file
- `ISDIR(string path) -> boolean` - Returns TRUE if the path is a directory

### Directory Operations

- `LISTDIR(string path) -> array` - Returns an array of strings containing the names of entries in the directory

### File Management

- `DELETE(string path) -> undefined` - Deletes a file or directory (recursively for directories)

## Examples

### Basic File Operations

```psy
IMPORT _FS
START

    // Write to a file
    WRITEFILE("hello.txt", "Hello, World!")
    OUTPUT "File created successfully"

    // Check if file exists
    IF EXISTS("hello.txt") THEN
        OUTPUT "File exists!"
    END

    // Read the file
    content = READFILE("hello.txt")
    OUTPUT "Content: ", content

    // Clean up
    DELETE("hello.txt")
    OUTPUT "File deleted"

END
```

### File System Explorer

```psy
IMPORT _FS[LISTDIR, ISFILE, ISDIR, EXISTS]
START

    // List current directory
    files = LISTDIR(".")
    
    OUTPUT "Directory contents:"
    OUTPUT files
    OUTPUT ""

    // Categorize entries
    OUTPUT "Files:"
    DECLARE ARRAY entries[10]
    
    OUTPUT "--- File System Explorer ---"
    
    // Check each entry type
    testPath = "modules"
    IF EXISTS(testPath) THEN
        IF ISDIR(testPath) THEN
            OUTPUT testPath, " is a directory"
        END
        IF ISFILE(testPath) THEN
            OUTPUT testPath, " is a file"
        END
    ELSE
        OUTPUT testPath, " does not exist"
    END

END
```

### Log File Creator

```psy
IMPORT _FS[READFILE, WRITEFILE, EXISTS]
START

    // Create a log file with timestamp
    logFile = "app.log"
    logEntry = ""
    
    // Read existing content if file exists
    IF EXISTS(logFile) THEN
        logEntry = READFILE(logFile)
        OUTPUT "Reading existing log file..."
    ELSE
        OUTPUT "Creating new log file..."
    END

    // Add new entry
    newEntry = "Application started at 2024-01-01 12:00:00\n"
    updatedLog = logEntry + newEntry
    
    WRITEFILE(logFile, updatedLog)
    OUTPUT "Log entry added"
    OUTPUT ""
    OUTPUT "Current log content:"
    OUTPUT READFILE(logFile)

END
```

### Configuration Manager

```psy
IMPORT _FS[READFILE, WRITEFILE, EXISTS, DELETE]
START

    configFile = "config.psy"
    
    // Check if config exists
    IF EXISTS(configFile) THEN
        OUTPUT "Config file found, reading..."
        config = READFILE(configFile)
        OUTPUT "Current config:"
        OUTPUT config
        OUTPUT ""
        
        // Backup config
        WRITEFILE("config.backup", config)
        OUTPUT "Backup created as config.backup"
    ELSE
        OUTPUT "No config file found, creating default..."
    END
    
    // Create/update configuration
    newConfig = "version = 1.0\n"
    newConfig = newConfig + "language = en\n"
    newConfig = newConfig + "theme = dark\n"
    
    WRITEFILE(configFile, newConfig)
    OUTPUT "Config saved!"
    OUTPUT ""
    OUTPUT "New config content:"
    OUTPUT READFILE(configFile)
    
    // Clean up backup
    IF EXISTS("config.backup") THEN
        DELETE("config.backup")
        OUTPUT "Backup deleted"
    END

END
```

### Batch File Processor

```psy
IMPORT _FS[LISTDIR, READFILE, WRITEFILE, ISFILE, EXISTS]
START

    // Process all text files in a directory
    OUTPUT "Processing .txt files in current directory..."
    OUTPUT ""
    
    allFiles = LISTDIR(".")
    OUTPUT "All files found: ", allFiles
    OUTPUT ""
    
    // For each file, check if it's a .txt file
    processedCount = 0
    
    // Process specific files
    testFile = "input.txt"
    
    IF EXISTS(testFile) AND ISFILE(testFile) THEN
        OUTPUT "Processing: ", testFile
        
        content = READFILE(testFile)
        OUTPUT "Original content:"
        OUTPUT content
        OUTPUT ""
        
        // Convert to uppercase
        uppercaseContent = "PROCESSED CONTENT: TEST DATA\n"
        processedFile = "output.txt"
        WRITEFILE(processedFile, uppercaseContent)
        
        OUTPUT "Processed and saved to: ", processedFile
        OUTPUT "Processed content:"
        OUTPUT READFILE(processedFile)
        
        processedCount = processedCount + 1
        OUTPUT ""
    END
    
    OUTPUT "Processed ", processedCount, " files"

END
```

### Project Structure Validator

```psy
IMPORT _FS[EXISTS, ISDIR, ISFILE, LISTDIR]
START

    OUTPUT "--- Project Validator ---"
    OUTPUT ""
    
    // Check required directories
    requiredDirs = ["modules", "core", "examples"]
    
    OUTPUT "Checking required directories..."
    allPresent = TRUE
    
    // Check modules directory
    IF EXISTS("modules") AND ISDIR("modules") THEN
        OUTPUT "✓ crates/ exists"
        
        // Check sub-modules
        submodules = LISTDIR("modules")
        OUTPUT "  Found modules: ", submodules
        
        // Check if fs module exists
        fsExists = EXISTS("crates/fs")
        IF fsExists AND ISDIR("crates/fs") THEN
            OUTPUT "  ✓ fs module found"
        ELSE
            OUTPUT "  ✗ fs module missing!"
            allPresent = FALSE
        END
        
        // Check if math module exists
        mathExists = EXISTS("crates/math")
        IF mathExists AND ISDIR("crates/math") THEN
            OUTPUT "  ✓ math module found"
        ELSE
            OUTPUT "  ✗ math module missing!"
            allPresent = FALSE
        END
    ELSE
        OUTPUT "✗ crates/ missing!"
        allPresent = FALSE
    END
    
    // Check core directory
    IF EXISTS("core") AND ISDIR("core") THEN
        OUTPUT "✓ core/ exists"
        
        coreFiles = LISTDIR("core")
        OUTPUT "  Core contents: ", coreFiles
    ELSE
        OUTPUT "✗ core/ missing!"
        allPresent = FALSE
    END
    
    // Check examples directory
    IF EXISTS("examples") AND ISDIR("examples") THEN
        OUTPUT "✓ examples/ exists"
        exampleCount = 0
        OUTPUT "  Examples available for testing"
    ELSE
        OUTPUT "✗ examples/ missing!"
        allPresent = FALSE
    END
    
    OUTPUT ""
    IF allPresent THEN
        OUTPUT "✓ All required directories present"
    ELSE
        OUTPUT "✗ Some required directories are missing"
    END

END
```

### Simple Text Editor

```psy
IMPORT _FS[READFILE, WRITEFILE, EXISTS, DELETE]
START

    filename = "notes.txt"
    
    // Create or edit a file
    IF EXISTS(filename) THEN
        OUTPUT "Opening existing file: ", filename
        currentContent = READFILE(filename)
        OUTPUT "Current content:"
        OUTPUT currentContent
        OUTPUT ""
        OUTPUT "Adding new line..."
        newLine = "New note added at line 4\n"
        updatedContent = currentContent + newLine
    ELSE
        OUTPUT "Creating new file: ", filename
        updatedContent = "My Notes File\n============\n\n"
        updatedContent = updatedContent + "First note created\n"
    END
    
    WRITEFILE(filename, updatedContent)
    OUTPUT "File saved successfully!"
    OUTPUT ""
    OUTPUT "Final content:"
    OUTPUT READFILE(filename)

END
```

## Error Handling

The fs module provides clear error messages when functions are used incorrectly:

```psy
IMPORT _FS
START

    // Wrong number of arguments
    result = READFILE()              // Error: READFILE expects 1 argument (path)
    result = WRITEFILE("test.txt")   // Error: WRITEFILE expects 2 arguments (path, content)

    // Wrong type
    result = READFILE(42)            // Error: READFILE expects a string path argument
    result = EXISTS(TRUE)            // Error: EXISTS expects a string path argument

    // File not found
    result = READFILE("nonexistent.txt")  // Error: Failed to read file 'nonexistent.txt'
    
    // Delete nonexistent path
    DELETE("nonexistent.txt")        // Error: Path 'nonexistent.txt' does not exist
    
    // Invalid directory
    result = LISTDIR("nonexistent/") // Error: Failed to list directory 'nonexistent/'

END
```

## Common Use Cases

### 1. Reading Configuration Files
```
IMPORT _FS[READFILE, EXISTS]

START
    IF EXISTS("config.txt") THEN
        config = READFILE("config.txt")
        // Parse and use configuration
    ELSE
        OUTPUT "No config file found, using defaults"
    END
END
```

### 2. Creating Reports
```
IMPORT _FS[WRITEFILE]

START
    report = "=== Daily Report ===\n"
    // ... generate report content
    WRITEFILE("report.txt", report)
END
```

### 3. File Management Scripts
```
IMPORT _FS[LISTDIR, EXISTS, DELETE, ISFILE]

START
    // Clean up temporary files
    files = LISTDIR("temp")
    // Process files...
END
```

### 4. Data Processing Pipelines
```
IMPORT _FS[READFILE, WRITEFILE, EXISTS]

START
    // Read input, process, write output
    data = READFILE("input.csv")
    // ... process data
    WRITEFILE("output.csv", processedData)
END
```

## Notes

- All file paths are relative to the current working directory
- `READFILE` reads the entire file content as a string
- `WRITEFILE` creates the file if it doesn't exist, or overwrites it if it does
- `DELETE` works on both files and directories (directories are deleted recursively)
- `LISTDIR` returns filenames only (not full paths)
- File operations may fail due to permissions, disk space, or other system limitations
- All functions validate input types before performing operations
- The module is synchronous (all operations complete before the next line executes)

## Security Considerations

- **Path Traversal**: Be cautious with user-provided file paths
- **File Permissions**: Operations respect the operating system's file permissions
- **Resource Limits**: Very large files may impact memory usage when using `READFILE`
- **Directory Deletion**: `DELETE` on directories is recursive and irreversible

## Performance Considerations

- All functions are implemented as native Rust functions for optimal performance
- `READFILE` loads entire file into memory - consider file size for large files
- `WRITEFILE` performs atomic writes when possible
- `LISTDIR` handles large directories efficiently
- Input validation occurs before any file system operation

## Contributing

To add new functions to the fs module:

1. Create a new file in `crates/fs/src/` (e.g., `copy_file.rs`)
2. Implement the function with signature: `pub fn copy_file(args: &[Value]) -> Result<Value, String>`
3. Register the function in `crates/fs/src/lib.rs`
4. Add the function to the `_FS` module in [core/src/interpreter/native.rs](../../core/src/interpreter/native.rs)
5. Update this README with the new function documentation

Example function structure:
```rust
// crates/fs/src/copy_file.rs
use types::Value;
use std::fs;

pub fn copy_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("COPYFILE expects 2 arguments (source, destination)".to_string());
    }
    
    let source = match &args[0] {
        Value::String(s) => s,
        _ => return Err("COPYFILE expects a string source path".to_string()),
    };
    
    let dest = match &args[1] {
        Value::String(s) => s,
        _ => return Err("COPYFILE expects a string destination path".to_string()),
    };
    
    match fs::copy(source, dest) {
        Ok(_) => Ok(Value::Undefined),
        Err(e) => Err(format!("Failed to copy '{}' to '{}': {}", source, dest, e)),
    }
}
```

## Planned Features

- `COPY(source, destination)` - Copy a file
- `MOVE(source, destination)` - Move/rename a file
- `MKDIR(path)` - Create a directory
- `FILE_SIZE(path)` - Get file size in bytes
- `FILE_MODIFIED(path)` - Get last modified timestamp
- `APPENDFILE(path, content)` - Append content to existing file

## License

This module is part of the Psy Interpreter project and is licensed under the same terms.

## See Also

- [Psy Interpreter Documentation](../../README.md)
- [Math Module Documentation](../math/README.md)
- [Other Built-in Modules](../../crates/)