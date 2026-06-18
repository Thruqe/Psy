IMPORT _ASYNC[AWAIT, FETCH, SLEEP, PARALLEL, PROMISE, TIMEOUT]

START
    OUTPUT "=== Async Prototype ==="
    OUTPUT ""
    
    // Basic sleep/delay
    OUTPUT "Starting delay..."
    AWAIT SLEEP(1000)  // Sleep for 1000ms
    OUTPUT "1 second passed"
    OUTPUT ""
    
    // Fetch data from URL
    url = "https://api.example.com/data"
    OUTPUT "Fetching data from: "
    OUTPUT url
    OUTPUT ""
    
    data = AWAIT FETCH(url)
    OUTPUT "Data received: "
    OUTPUT data
    OUTPUT ""
    
    // Parallel execution
    OUTPUT "Running tasks in parallel..."
    results = AWAIT PARALLEL(
        FETCH("https://api.example.com/user"),
        FETCH("https://api.example.com/posts"),
        FETCH("https://api.example.com/comments")
    )
    
    OUTPUT "Parallel results: "
    OUTPUT results
    OUTPUT ""
    
    // Promise with then/catch
    OUTPUT "Promise example..."
    promise = PROMISE(FUNCTION() 
        SLEEP(500)
        RETURN "Promise resolved"
    ENDFUNCTION)
    
    result = AWAIT promise
    OUTPUT "Promise result: "
    OUTPUT result
    OUTPUT ""
    
    // Timeout example
    OUTPUT "Timeout example (2 seconds)..."
    result = AWAIT TIMEOUT(
        FETCH("https://api.example.com/slow"),
        2000  // Timeout after 2 seconds
    )
    
    IF result == "TIMEOUT" THEN
        OUTPUT "Operation timed out"
    ELSE
        OUTPUT "Result: "
        OUTPUT result
    ENDIF
END