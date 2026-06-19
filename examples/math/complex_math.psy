START
    /* ============================================================
       SECTION 1: NEWTON-RAPHSON ROOT FINDER
       Finds the root of f(x) = x^3 - 2x - 5 using iterative method
       ============================================================ */
    
    OUTPUT "=== Newton-Raphson Root Finder ==="
    OUTPUT "Solving: x^3 - 2x - 5 = 0"
    OUTPUT ""
    
    // Initialize variables for the root-finding algorithm
    x = 2.0                  // Initial guess for the root
    tolerance = 0.00001      // Convergence threshold
    maxIterations = 100      // Maximum iterations to prevent infinite loops
    iter = 0                 // Iteration counter
    converged = FALSE        // Flag to indicate if solution found
    
    // Main Newton-Raphson iteration loop
    WHILE iter < maxIterations AND NOT converged
        // Evaluate the function f(x) at current x
        fx = x^3 - 2*x - 5
        
        // Evaluate the derivative f'(x) at current x
        fprime = 3*x^2 - 2
        
        // Check if derivative is too close to zero (prevents division by zero)
        IF fprime < 0.00001 AND fprime > -0.00001 THEN
            OUTPUT "Derivative too small at x = "
            OUTPUT x
            converged = TRUE
        ELSE
            // Newton-Raphson update formula: x_new = x - f(x)/f'(x)
            xNew = x - fx / fprime
            
            // Check if change is within tolerance (convergence)
            IF xNew - x < tolerance AND xNew - x > -tolerance THEN
                converged = TRUE
            ENDIF
            
            // Update x for next iteration
            x = xNew
            iter = iter + 1
        ENDIF
    ENDWHILE
    
    // Display the results of the root-finding
    OUTPUT ""
    OUTPUT "=== Results ==="
    OUTPUT "Root found at x = "
    OUTPUT x
    OUTPUT "Iterations: "
    OUTPUT iter
    
    // Verify the solution by evaluating f(x) and f'(x) at the found root
    fx = x^3 - 2*x - 5
    OUTPUT "f(x) = "
    OUTPUT fx
    OUTPUT "f'(x) = "
    OUTPUT 3*x^2 - 2
    
    /* ============================================================
       SECTION 2: MATRIX OPERATIONS
       Demonstrates 2x2 matrix addition and multiplication
       ============================================================ */
    
    OUTPUT ""
    OUTPUT "=== Matrix Operations ==="
    
    // Declare 2x2 matrices as 1D arrays (row-major order)
    DECLARE A[4]    // Matrix A = [[1, 2], [3, 4]]
    DECLARE B[4]    // Matrix B = [[5, 6], [7, 8]]
    DECLARE C[4]    // Matrix C for addition result
    
    // Initialize matrix A
    A[0] = 1     // Row 0, Col 0
    A[1] = 2     // Row 0, Col 1
    A[2] = 3     // Row 1, Col 0
    A[3] = 4     // Row 1, Col 1
    
    // Initialize matrix B
    B[0] = 5     // Row 0, Col 0
    B[1] = 6     // Row 0, Col 1
    B[2] = 7     // Row 1, Col 0
    B[3] = 8     // Row 1, Col 1
    
    // Display matrix A
    OUTPUT "Matrix A:"
    OUTPUT A[0]
    OUTPUT " "
    OUTPUT A[1]
    OUTPUT ""
    OUTPUT A[2]
    OUTPUT " "
    OUTPUT A[3]
    OUTPUT ""
    
    // Display matrix B
    OUTPUT "Matrix B:"
    OUTPUT B[0]
    OUTPUT " "
    OUTPUT B[1]
    OUTPUT ""
    OUTPUT B[2]
    OUTPUT " "
    OUTPUT B[3]
    OUTPUT ""
    
    // Matrix addition: C = A + B (element-wise)
    C[0] = A[0] + B[0]
    C[1] = A[1] + B[1]
    C[2] = A[2] + B[2]
    C[3] = A[3] + B[3]
    
    // Display addition result
    OUTPUT "Matrix Addition (A + B):"
    OUTPUT C[0]
    OUTPUT " "
    OUTPUT C[1]
    OUTPUT ""
    OUTPUT C[2]
    OUTPUT " "
    OUTPUT C[3]
    OUTPUT ""
    
    // Matrix multiplication: D = A * B
    DECLARE D[4]
    // Compute each element using dot product of rows and columns
    D[0] = A[0]*B[0] + A[1]*B[2]    // Row 0, Col 0
    D[1] = A[0]*B[1] + A[1]*B[3]    // Row 0, Col 1
    D[2] = A[2]*B[0] + A[3]*B[2]    // Row 1, Col 0
    D[3] = A[2]*B[1] + A[3]*B[3]    // Row 1, Col 1
    
    // Display multiplication result
    OUTPUT "Matrix Multiplication (A * B):"
    OUTPUT D[0]
    OUTPUT " "
    OUTPUT D[1]
    OUTPUT ""
    OUTPUT D[2]
    OUTPUT " "
    OUTPUT D[3]
    OUTPUT ""
    
    /* ============================================================
       SECTION 3: NUMERICAL INTEGRATION
       Uses the trapezoidal rule to approximate a definite integral
       ============================================================ */
    
    OUTPUT ""
    OUTPUT "=== Numerical Integration ==="
    
    // Define integration parameters
    a = 0          // Lower limit of integration
    b = 10         // Upper limit of integration
    n = 100        // Number of subintervals (higher = more accurate)
    h = (b - a) / n    // Step size (width of each subinterval)
    
    // Compute sum of f(x) for interior points
    sum = 0
    FOR i = 1 TO n-1
        x = a + i*h                  // Current x-coordinate
        sum = sum + x^2              // Add f(x) = x^2 to sum
    ENDFOR
    
    // Trapezoidal rule formula: 
    // ∫f(x)dx ≈ (h/2) * [f(a) + f(b) + 2*Σf(x_i)]
    integral = (h/2) * (a^2 + b^2 + 2*sum)
    
    // Display integration results
    OUTPUT "Trapezoidal Rule Integration"
    OUTPUT "Integral of x^2 from 0 to 10"
    OUTPUT "Result: "
    OUTPUT integral
    
    /* ============================================================
       SECTION 4: STATISTICAL ANALYSIS
       Computes mean, variance, and standard deviation
       ============================================================ */
    
    OUTPUT ""
    OUTPUT "=== Statistical Analysis ==="
    
    // Declare and initialize the dataset
    DECLARE data[6]
    data[0] = 12
    data[1] = 15
    data[2] = 18
    data[3] = 22
    data[4] = 27
    data[5] = 30
    
    OUTPUT "Data set: 12, 15, 18, 22, 27, 30"
    
    // Calculate the arithmetic mean (average)
    total = 0
    n = 6                // Number of data points
    FOR i = 0 TO n-1
        total = total + data[i]    // Sum all data values
    ENDFOR
    
    mean = total / n               // Mean = sum / count
    OUTPUT "Mean: "
    OUTPUT mean
    
    // Calculate variance and standard deviation
    sumSquares = 0
    FOR i = 0 TO n-1
        diff = data[i] - mean                // Deviation from mean
        sumSquares = sumSquares + diff^2     // Sum of squared deviations
    ENDFOR
    
    // Population variance = average of squared deviations
    variance = sumSquares / n
    // Standard deviation = square root of variance
    stdDev = variance^0.5
    
    // Display statistical results
    OUTPUT "Variance: "
    OUTPUT variance
    OUTPUT "Standard Deviation: "
    OUTPUT stdDev
END