IMPORT _MATH[MEAN, MEDIAN, MODE, VARIANCE, STDDEV, MIN, MAX, SUM, PRODUCT]

START
    // Data array
    DECLARE data[7]
    data[0] = 12
    data[1] = 15
    data[2] = 18
    data[3] = 22
    data[4] = 27
    data[5] = 30
    data[6] = 35
    
    // Statistical functions
    result = SUM(data)
    OUTPUT "SUM: "
    OUTPUT result
    OUTPUT ""
    
    result = MEAN(data)
    OUTPUT "MEAN: "
    OUTPUT result
    OUTPUT ""
    
    result = MEDIAN(data)
    OUTPUT "MEDIAN: "
    OUTPUT result
    OUTPUT ""
    
    result = MODE(data)
    OUTPUT "MODE: "
    OUTPUT result
    OUTPUT ""
    
    result = VARIANCE(data)
    OUTPUT "VARIANCE: "
    OUTPUT result
    OUTPUT ""
    
    result = STDDEV(data)
    OUTPUT "STDDEV: "
    OUTPUT result
    OUTPUT ""
    
    result = MIN(data)
    OUTPUT "MIN: "
    OUTPUT result
    OUTPUT ""
    
    result = MAX(data)
    OUTPUT "MAX: "
    OUTPUT result
    OUTPUT ""
    
    result = PRODUCT(data)
    OUTPUT "PRODUCT: "
    OUTPUT result
END