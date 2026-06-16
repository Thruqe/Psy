# Pseudocode Math Module

A built-in math module for the Pseudocode Interpreter providing common mathematical functions and constants.

## Overview

The `_MATH` module provides a comprehensive set of mathematical functions and constants for use in pseudocode programs. All trigonometric functions operate with **degrees** (not radians) for educational purposes and ease of use.

## Importing the Module

To use the math module in your pseudocode program:

```pseudocode
IMPORT _MATH
```

To import only specific functions:

```pseudocode
IMPORT _MATH[SIN, COS, TAN]
```

## Available Functions

### Trigonometric Functions (Degrees)

| Function | Description          | Example   | Result |
| -------- | -------------------- | --------- | ------ |
| `SIN(x)` | Sine of x degrees    | `SIN(30)` | `0.5`  |
| `COS(x)` | Cosine of x degrees  | `COS(60)` | `0.5`  |
| `TAN(x)` | Tangent of x degrees | `TAN(45)` | `1.0`  |

### Basic Mathematical Functions

| Function    | Description    | Example     | Result |
| ----------- | -------------- | ----------- | ------ |
| `ABS(x)`    | Absolute value | `ABS(-5)`   | `5`    |
| `SQRT(x)`   | Square root    | `SQRT(16)`  | `4`    |
| `POW(x, y)` | x raised to y  | `POW(2, 3)` | `8`    |

### Rounding Functions

| Function   | Description                    | Example      | Result |
| ---------- | ------------------------------ | ------------ | ------ |
| `ROUND(x)` | Rounds to nearest integer      | `ROUND(3.7)` | `4`    |
| `FLOOR(x)` | Rounds down to nearest integer | `FLOOR(3.7)` | `3`    |
| `CEIL(x)`  | Rounds up to nearest integer   | `CEIL(3.2)`  | `4`    |

## Available Constants

| Constant | Description    | Value               |
| -------- | -------------- | ------------------- |
| `PI`     | Pi (π)         | `3.141592653589793` |
| `E`      | Euler's number | `2.718281828459045` |

## Examples

### Basic Math Operations

```pseudocode
IMPORT _MATH
START

    // Trigonometric calculations
    angle = 45
    sin_value = SIN(angle)
    cos_value = COS(angle)
    tan_value = TAN(angle)

    OUTPUT "Angle: ", angle
    OUTPUT "SIN: ", sin_value
    OUTPUT "COS: ", cos_value
    OUTPUT "TAN: ", tan_value

    // Using constants
    area = PI * POW(5, 2)   // Area of circle with radius 5
    OUTPUT "Area of circle: ", area

    // Rounding
    num = 3.14159
    OUTPUT "Original: ", num
    OUTPUT "Rounded: ", ROUND(num)
    OUTPUT "Floored: ", FLOOR(num)
    OUTPUT "Ceiled: ", CEIL(num)

END
```

### Advanced Math Example

```pseudocode
IMPORT _MATH[POW, SQRT, COS, SIN]
START

    // Calculate distance between two points
    x1 = 0
    y1 = 0
    x2 = 3
    y2 = 4

    dx = x2 - x1
    dy = y2 - y1
    distance = SQRT(POW(dx, 2) + POW(dy, 2))

    OUTPUT "Distance between points: ", distance

    // Pythagorean theorem
    a = 3
    b = 4
    c = SQRT(POW(a, 2) + POW(b, 2))
    OUTPUT "Hypotenuse: ", c

    // Law of cosines
    side_a = 5
    side_b = 6
    angle_c = 60
    side_c = SQRT(POW(side_a, 2) + POW(side_b, 2) - 2 * side_a * side_b * COS(angle_c))
    OUTPUT "Side c: ", side_c

END
```

### Geometry Calculator

```pseudocode
IMPORT _MATH[PI, SIN, COS, TAN, POW, SQRT]
START

    // Circle calculations
    radius = 5
    circumference = 2 * PI * radius
    area = PI * POW(radius, 2)

    OUTPUT "Circle with radius: ", radius
    OUTPUT "Circumference: ", circumference
    OUTPUT "Area: ", area

    // Triangle calculations
    angle = 30
    opposite = 10
    hypotenuse = opposite / SIN(angle)
    adjacent = opposite / TAN(angle)

    OUTPUT "Triangle:"
    OUTPUT "  Angle: ", angle, "°"
    OUTPUT "  Opposite: ", opposite
    OUTPUT "  Adjacent: ", adjacent
    OUTPUT "  Hypotenuse: ", hypotenuse

END
```

### Statistical Calculations

```pseudocode
IMPORT _MATH[POW, SQRT, ABS, ROUND]
START

    // Calculate standard deviation
    DECLARE ARRAY data[5]
    data[0] = 2
    data[1] = 4
    data[2] = 6
    data[3] = 8
    data[4] = 10

    // Calculate mean
    sum = 0
    FOR i = 0 TO 4
        sum = sum + data[i]
    ENDFOR
    mean = sum / 5

    // Calculate variance
    sum_sq_diff = 0
    FOR i = 0 TO 4
        diff = data[i] - mean
        sum_sq_diff = sum_sq_diff + POW(diff, 2)
    ENDFOR
    variance = sum_sq_diff / 5
    std_dev = SQRT(variance)

    OUTPUT "Mean: ", mean
    OUTPUT "Standard Deviation: ", ROUND(std_dev * 100) / 100

END
```

## Error Handling

The math module provides clear error messages when functions are used incorrectly:

```pseudocode
IMPORT _MATH
START

    // Wrong number of arguments
    result = SIN()           // Error: SIN expects 1 argument

    // Wrong type
    result = SIN("string")   // Error: SIN expects a number

    // Domain errors
    result = SQRT(-1)        // Error: SQRT of negative number
    result = TAN(90)         // Undefined result (near infinity)

END
```

## Notes

- All angles in trigonometric functions are in **degrees**
- Floating-point precision may result in values like `0.5000000000000001` instead of exactly `0.5`
- Constants `PI` and `E` are available for use in calculations
- The module supports both integer and floating-point inputs

## Performance Considerations

- All functions are implemented as native Rust functions for optimal performance
- Input validation is performed on all functions to prevent runtime errors
- Memory-efficient operation with no unnecessary allocations

## Contributing

To add new functions to the math module:

1. Create a new file in `modules/math/src/` (e.g., `new_func.rs`)
2. Implement the function with signature: `pub fn new_func(args: &[Value]) -> Result<Value, String>`
3. Register the function in `modules/math/src/lib.rs`
4. Add the function to the `_MATH` module in [Here](../../core/src/interpreter/native.rs)

## License

This module is part of the Pseudocode Interpreter project and is licensed under the same terms.

## See Also

- [Pseudocode Interpreter Documentation](../../README.md)
- [Other Built-in Modules](../../modules/)
