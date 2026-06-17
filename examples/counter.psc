START
    FUNCTION test(flag)
        IF flag THEN
            STATIC hits = 0
            hits = hits + 1
            OUTPUT hits
        ENDIF
    ENDFUNCTION

    test(true)
    test(false)
    test(true)
END