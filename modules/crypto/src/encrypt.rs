use aes::Aes256;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Decryptor, Encryptor};
use psy_types::Value;
use sha2::{Digest, Sha256};

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

fn derive_key_iv(password: &str) -> ([u8; 32], [u8; 16]) {
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];

    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let hash = hasher.finalize();
    key.copy_from_slice(&hash[..32]);

    let mut hasher2 = Sha256::new();
    hasher2.update(&hash);
    hasher2.update(b"iv");
    let hash2 = hasher2.finalize();
    iv.copy_from_slice(&hash2[..16]);

    (key, iv)
}

pub fn encrypt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("ENCRYPT expects 2 arguments (text, password)".to_string());
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err("ENCRYPT expects a string as first argument".to_string()),
    };

    let password = match &args[1] {
        Value::String(s) => s,
        _ => return Err("ENCRYPT expects a string password as second argument".to_string()),
    };

    let (key, iv) = derive_key_iv(password);

    // Pad the plaintext to multiple of 16 bytes
    let mut data = text.as_bytes().to_vec();
    let padding_len = 16 - (data.len() % 16);
    data.extend(vec![padding_len as u8; padding_len]);

    // Create buffer with enough capacity
    let mut buffer = vec![0u8; data.len()];
    buffer.copy_from_slice(&data);

    // Encrypt
    let encryptor = Aes256CbcEnc::new(&key.into(), &iv.into());
    let encrypted = encryptor
        .encrypt_padded_mut::<cbc::cipher::block_padding::NoPadding>(&mut buffer, data.len())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Return as base64 for safe string handling
    use base64::Engine;
    Ok(Value::String(
        base64::engine::general_purpose::STANDARD.encode(encrypted),
    ))
}

pub fn decrypt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("DECRYPT expects 2 arguments (encrypted_text, password)".to_string());
    }

    let encrypted = match &args[0] {
        Value::String(s) => s,
        _ => return Err("DECRYPT expects a string as first argument".to_string()),
    };

    let password = match &args[1] {
        Value::String(s) => s,
        _ => return Err("DECRYPT expects a string password as second argument".to_string()),
    };

    let (key, iv) = derive_key_iv(password);

    // Decode base64
    use base64::Engine;
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Invalid base64 input: {}", e))?;

    // Create buffer
    let mut buffer = ciphertext.clone();

    // Decrypt
    let decryptor = Aes256CbcDec::new(&key.into(), &iv.into());
    let plaintext = decryptor
        .decrypt_padded_mut::<cbc::cipher::block_padding::NoPadding>(&mut buffer)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    // Remove PKCS7 padding
    if plaintext.is_empty() {
        return Err("Decryption failed: empty data".to_string());
    }

    let padding_len = plaintext[plaintext.len() - 1] as usize;
    if padding_len == 0 || padding_len > 16 || plaintext.len() < padding_len {
        return Err("Decryption failed: invalid padding".to_string());
    }

    let unpadded = &plaintext[..plaintext.len() - padding_len];

    String::from_utf8(unpadded.to_vec())
        .map(Value::String)
        .map_err(|e| format!("Invalid UTF-8 in decrypted text: {}", e))
}
