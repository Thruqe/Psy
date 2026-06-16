START
    OUTPUT "How many scores do you want to enter?"
    INPUT count
    
    total = 0
    FOR i = 1 TO count
        OUTPUT "Enter score "
        OUTPUT i
        OUTPUT ": "
        INPUT score
        total = total + score
    ENDFOR
    
    average = total / count
    OUTPUT "The average of "
    OUTPUT count
    OUTPUT " scores is: "
    OUTPUT average
END