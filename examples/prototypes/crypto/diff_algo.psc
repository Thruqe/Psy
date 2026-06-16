IMPORT _CRYPTO[AES_ENCRYPT, AES_DECRYPT, RSA_ENCRYPT, RSA_DECRYPT, RSA_GENERATE_KEY]

START
    // AES variables
    data = "Secret message"
    aesKey = "01234567890123456789012345678901"
    aesIv = "1234567890123456"
    aesEncrypted = ""
    aesDecrypted = ""
    
    // RSA variables
    rsaKeys = ""
    rsaEncrypted = ""
    rsaDecrypted = ""
    
    // AES Encryption
    aesEncrypted = AES_ENCRYPT(data, aesKey, aesIv)
    aesDecrypted = AES_DECRYPT(aesEncrypted, aesKey, aesIv)
    
    // RSA Key Generation
    rsaKeys = RSA_GENERATE_KEY()
    
    // RSA Encryption
    rsaEncrypted = RSA_ENCRYPT(data, rsaKeys.public)
    rsaDecrypted = RSA_DECRYPT(rsaEncrypted, rsaKeys.private)
    
    // Output
    OUTPUT "AES Decrypted: "
    OUTPUT aesDecrypted
    OUTPUT ""
    OUTPUT "RSA Decrypted: "
    OUTPUT rsaDecrypted
END