START
    FUNCTION recurse(n)
        STATIC calls = 0
        calls = calls + 1
        IF n > 0 THEN
            recurse(n - 1)
        ENDIF
        RETURN calls
    ENDFUNCTION

    OUTPUT recurse(3)
END