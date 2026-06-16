START
    DECLARE scores[5]
    scores[0] = 85
    scores[1] = 72
    scores[2] = 90
    scores[3] = 68
    scores[4] = 88
    total = 0
    FOR i = 0 TO 4
        total = total + scores[i]
    ENDFOR
    OUTPUT "Total: "
    OUTPUT total
END