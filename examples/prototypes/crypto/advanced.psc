IMPORT _CRYPTO[ENCRYPT, DECRYPT, HASH, BASE64_ENCODE, BASE64_DECODE, HMAC_GENERATE, HMAC_VERIFY]

START
    // Test data
    text = "Test message"
    password = "password123"
    salt = "salt456"
    
    // Encrypt
    encrypted = ENCRYPT(text, password)
    
    // Decrypt
    decrypted = DECRYPT(encrypted, password)
    
    // Hash with salt
    hashed = HASH(text + salt)
    
    // Base64
    encoded = BASE64_ENCODE(text)
    decoded = BASE64_DECODE(encoded)
    
    // HMAC
    hmac = HMAC_GENERATE(text, password)
    verified = HMAC_VERIFY(text, hmac, password)
    
    // Output all results
    OUTPUT encrypted
    OUTPUT ""
    OUTPUT decrypted
    OUTPUT ""
    OUTPUT hashed
    OUTPUT ""
    OUTPUT encoded
    OUTPUT ""
    OUTPUT decoded
    OUTPUT ""
    OUTPUT hmac
    OUTPUT ""
    OUTPUT verified
END