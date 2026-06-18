# Psy Math Module

A built-in math module for the Psy Interpreter providing common mathematical functions and constants.

## Overview

The `_MATH` module provides a comprehensive set of mathematical functions and constants for use in psy programs. All trigonometric functions operate with **degrees** (not radians) for educational purposes and ease of use.

## Importing the Module

To use the math module in your psy program:

```psy
IMPORT _MATH
```

To import only specific functions:

```psy
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

### Statistical Functions

| Function        | Description               | Example                   | Result               |
| --------------- | ------------------------- | ------------------------- | -------------------- |
| `MEAN(...)`     | Arithmetic mean (average) | `MEAN(1, 2, 3, 4, 5)`     | `3`                  |
| `MEDIAN(...)`   | Middle value              | `MEDIAN(1, 2, 3, 4, 5)`   | `3`                  |
| `MODE(...)`     | Most frequent value(s)    | `MODE(1, 2, 2, 3, 4)`     | `2`                  |
| `VARIANCE(...)` | Variance of data set      | `VARIANCE(1, 2, 3, 4, 5)` | `2`                  |
| `STDDEV(...)`   | Standard deviation        | `STDDEV(1, 2, 3, 4, 5)`   | `1.4142135623730951` |
| `MIN(...)`      | Minimum value             | `MIN(1, 2, 3, 4, 5)`      | `1`                  |
| `MAX(...)`      | Maximum value             | `MAX(1, 2, 3, 4, 5)`      | `5`                  |
| `SUM(...)`      | Sum of values             | `SUM(1, 2, 3, 4, 5)`      | `15`                 |
| `PRODUCT(...)`  | Product of values         | `PRODUCT(1, 2, 3, 4, 5)`  | `120`                |

### Matrix Operations

| Function                     | Description       | Example                   | Result          |
| ---------------------------- | ----------------- | ------------------------- | --------------- |
| `MATRIX(rows, cols, values)` | Create a matrix   | `MATRIX(2, 2, [1,2,3,4])` | `[[1,2],[3,4]]` |
| `MATRIX_ADD(A, B)`           | Add two matrices  | `MATRIX_ADD(A, B)`        | Matrix sum      |
| `MATRIX_MULTIPLY(A, B)`      | Multiply matrices | `MATRIX_MULTIPLY(A, B)`   | Matrix product  |

### Vector Operations

| Function               | Description     | Example                          | Result    |
| ---------------------- | --------------- | -------------------------------- | --------- |
| `VECTOR(...)`          | Create a vector | `VECTOR(1, 2, 3)`                | `[1,2,3]` |
| `VECTOR_DOT(v1, v2)`   | Dot product     | `VECTOR_DOT([1,2], [3,4])`       | `11`      |
| `VECTOR_CROSS(v1, v2)` | Cross product   | `VECTOR_CROSS([1,0,0], [0,1,0])` | `[0,0,1]` |

**Note:** All statistical functions accept both individual numbers and arrays. For example:

```psy
MEAN(1, 2, 3, 4, 5)     // Returns 3
MEAN(data_array)         // Returns mean of array elements
MEAN(1, 2, data_array)   // Mixed input is also supported
```

## Available Constants

| Constant | Description    | Value               |
| -------- | -------------- | ------------------- |
| `PI`     | Pi (π)         | `3.141592653589793` |
| `E`      | Euler's number | `2.718281828459045` |

## Function Signatures

### Trigonometry

- `SIN(number angle) -> number` - Sine of angle in degrees
- `COS(number angle) -> number` - Cosine of angle in degrees
- `TAN(number angle) -> number` - Tangent of angle in degrees

### Statistics

- `MEAN(number|array ...) -> number` - Mean of all arguments
- `MEDIAN(number|array ...) -> number` - Median value
- `MODE(number|array ...) -> number|array` - Most frequent value(s)

## Examples

### Basic Math Operations

```psy
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

```psy
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

```psy
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

```psy
IMPORT _MATH[MEAN, MEDIAN, MODE, VARIANCE, STDDEV, MIN, MAX, SUM, PRODUCT]
START

    // Create a data set
    DECLARE ARRAY data[7]
    data[0] = 2
    data[1] = 4
    data[2] = 6
    data[3] = 8
    data[4] = 10
    data[5] = 12
    data[6] = 14

    OUTPUT "Data: [2, 4, 6, 8, 10, 12, 14]"
    OUTPUT "Sum: ", SUM(data)
    OUTPUT "Product: ", PRODUCT(2, 4, 6)
    OUTPUT "Mean: ", MEAN(data)
    OUTPUT "Median: ", MEDIAN(data)
    OUTPUT "Min: ", MIN(data)
    OUTPUT "Max: ", MAX(data)
    OUTPUT "Variance: ", VARIANCE(data)
    OUTPUT "Standard Deviation: ", STDDEV(data)

    // Mode example with repeated values
    DECLARE ARRAY scores[6]
    scores[0] = 85
    scores[1] = 90
    scores[2] = 85
    scores[3] = 95
    scores[4] = 90
    scores[5] = 85

    OUTPUT "Scores: [85, 90, 85, 95, 90, 85]"
    OUTPUT "Mode: ", MODE(scores)

END
```

### Data Analysis Example

```psy
IMPORT _MATH[MEAN, MEDIAN, MODE, VARIANCE, STDDEV, MIN, MAX]
START

    // Analyze test scores
    DECLARE ARRAY scores[10]
    scores[0] = 85
    scores[1] = 92
    scores[2] = 78
    scores[3] = 95
    scores[4] = 88
    scores[5] = 76
    scores[6] = 91
    scores[7] = 84
    scores[8] = 97
    scores[9] = 89

    OUTPUT "--- Test Score Analysis ---"
    OUTPUT "Scores: [85, 92, 78, 95, 88, 76, 91, 84, 97, 89]"
    OUTPUT "Mean: ", MEAN(scores)
    OUTPUT "Median: ", MEDIAN(scores)
    OUTPUT "Range: ", MIN(scores), " - ", MAX(scores)
    OUTPUT "Variance: ", VARIANCE(scores)
    OUTPUT "Std Dev: ", STDDEV(scores)

    // Add a new score and recalculate
    scores[9] = 100
    OUTPUT "Updated score: 100"
    OUTPUT "New Mean: ", MEAN(scores)
    OUTPUT "New Std Dev: ", STDDEV(scores)

END
```

## Trigonometric identity verification:

````psy
IMPORT _MATH[SIN, COS, TAN, PI]
START
    angle = 45
    // Verify sin²θ + cos²θ = 1
    identity = POW(SIN(angle), 2) + POW(COS(angle), 2)
    OUTPUT "sin²(45°) + cos²(45°) = ", identity  // Should output 1
END

## Error Handling

The math module provides clear error messages when functions are used incorrectly:

```psy
IMPORT _MATH
START

    // Wrong number of arguments
    result = SIN()           // Error: SIN expects 1 argument

    // Wrong type
    result = SIN("string")   // Error: SIN expects a number

    // Domain errors
    result = SQRT(-1)        // Error: SQRT of negative number
    result = TAN(90)         // Undefined result (near infinity)

    // Statistical function errors
    result = MEAN()          // Error: MEAN expects at least 1 argument
    result = MEAN("text")    // Error: MEAN expects numbers or arrays of numbers

END
````

## Notes

- All angles in trigonometric functions are in **degrees**
- Floating-point precision may result in values like `0.5000000000000001` instead of exactly `0.5`
- Constants `PI` and `E` are available for use in calculations
- The module supports both integer and floating-point inputs
- Statistical functions accept both individual arguments and arrays
- `MODE` returns an array if there are multiple modes, or a single number if there's one mode

## Performance Considerations

- All functions are implemented as native Rust functions for optimal performance
- Input validation is performed on all functions to prevent runtime errors
- Memory-efficient operation with no unnecessary allocations
- Statistical functions handle large datasets efficiently

## Contributing

To add new functions to the math module:

1. Create a new file in `modules/math/src/` (e.g., `new_func.rs`)
2. Implement the function with signature: `pub fn new_func(args: &[Value]) -> Result<Value, String>`
3. Register the function in `modules/math/src/lib.rs`
4. Add the function to the `_MATH` module in [core/src/interpreter/native.rs](../../core/src/interpreter/native.rs)

## License

This module is part of the Psy Interpreter project and is licensed under the same terms.

## See Also

- [Psy Interpreter Documentation](../../README.md)
- [Other Built-in Modules](../../modules/)
