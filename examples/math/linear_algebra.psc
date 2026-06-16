IMPORT _MATH[MATRIX_ADD, MATRIX_MULTIPLY, MATRIX_TRANSPOSE, MATRIX_DETERMINANT, MATRIX_INVERSE, DOT, CROSS]

START
    // 2x2 matrices
    DECLARE A[4]
    DECLARE B[4]
    DECLARE C[4]
    
    A[0] = 1
    A[1] = 2
    A[2] = 3
    A[3] = 4
    
    B[0] = 5
    B[1] = 6
    B[2] = 7
    B[3] = 8
    
    // Matrix addition
    C = MATRIX_ADD(A, B)
    OUTPUT "Matrix Addition: "
    OUTPUT C[0]
    OUTPUT " "
    OUTPUT C[1]
    OUTPUT ""
    OUTPUT C[2]
    OUTPUT " "
    OUTPUT C[3]
    OUTPUT ""
    
    // Matrix multiplication
    C = MATRIX_MULTIPLY(A, B)
    OUTPUT "Matrix Multiplication: "
    OUTPUT C[0]
    OUTPUT " "
    OUTPUT C[1]
    OUTPUT ""
    OUTPUT C[2]
    OUTPUT " "
    OUTPUT C[3]
    OUTPUT ""
    
    // Matrix transpose
    C = MATRIX_TRANSPOSE(A)
    OUTPUT "Matrix Transpose: "
    OUTPUT C[0]
    OUTPUT " "
    OUTPUT C[1]
    OUTPUT ""
    OUTPUT C[2]
    OUTPUT " "
    OUTPUT C[3]
    OUTPUT ""
    
    // Determinant
    det = MATRIX_DETERMINANT(A)
    OUTPUT "Determinant: "
    OUTPUT det
    OUTPUT ""
    
    // Vector operations
    DECLARE V1[3]
    DECLARE V2[3]
    DECLARE V3[3]
    
    V1[0] = 1
    V1[1] = 2
    V1[2] = 3
    
    V2[0] = 4
    V2[1] = 5
    V2[2] = 6
    
    // Dot product
    dot = DOT(V1, V2)
    OUTPUT "Dot Product: "
    OUTPUT dot
    OUTPUT ""
    
    // Cross product
    V3 = CROSS(V1, V2)
    OUTPUT "Cross Product: "
    OUTPUT V3[0]
    OUTPUT " "
    OUTPUT V3[1]
    OUTPUT " "
    OUTPUT V3[2]
END