IMPORT _MATH[SIN, COS, TAN, SQRT, POW, ABS, ROUND, FLOOR, CEIL, PI, E]

START
    // Variables
    x = 45
    y = 9
    z = 3.7
    result = 0
    
    // Trigonometric functions (in radians)
    result = SIN(x * PI / 180)
    OUTPUT "SIN(45°): "
    OUTPUT result
    OUTPUT ""
    
    result = COS(x * PI / 180)
    OUTPUT "COS(45°): "
    OUTPUT result
    OUTPUT ""
    
    result = TAN(x * PI / 180)
    OUTPUT "TAN(45°): "
    OUTPUT result
    OUTPUT ""
    
    // Square root
    result = SQRT(y)
    OUTPUT "SQRT(9): "
    OUTPUT result
    OUTPUT ""
    
    // Power
    result = POW(2, 10)
    OUTPUT "2^10: "
    OUTPUT result
    OUTPUT ""
    
    // Absolute value
    result = ABS(-42)
    OUTPUT "ABS(-42): "
    OUTPUT result
    OUTPUT ""
    
    // Rounding
    result = ROUND(z)
    OUTPUT "ROUND(3.7): "
    OUTPUT result
    OUTPUT ""
    
    result = FLOOR(z)
    OUTPUT "FLOOR(3.7): "
    OUTPUT result
    OUTPUT ""
    
    result = CEIL(z)
    OUTPUT "CEIL(3.7): "
    OUTPUT result
    OUTPUT ""
    
    // Constants
    OUTPUT "PI: "
    OUTPUT PI
    OUTPUT ""
    
    OUTPUT "E: "
    OUTPUT E
END