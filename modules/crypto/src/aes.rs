use aes::Aes256;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Decryptor, Encryptor};
use psy_types::Value;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

pub fn aes_encrypt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("AES_ENCRYPT expects 3 arguments (text, key, iv)".to_string());
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err("AES_ENCRYPT expects a string as first argument".to_string()),
    };

    let key_str = match &args[1] {
        Value::String(s) => s,
        _ => return Err("AES_ENCRYPT expects a string key as second argument".to_string()),
    };

    let iv_str = match &args[2] {
        Value::String(s) => s,
        _ => return Err("AES_ENCRYPT expects a string IV as third argument".to_string()),
    };

    if key_str.len() != 32 {
        return Err("AES_ENCRYPT key must be 32 characters (256 bits)".to_string());
    }
    if iv_str.len() != 16 {
        return Err("AES_ENCRYPT IV must be 16 characters (128 bits)".to_string());
    }

    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];
    key.copy_from_slice(key_str.as_bytes());
    iv.copy_from_slice(iv_str.as_bytes());

    // Pad data
    let mut data = text.as_bytes().to_vec();
    let padding_len = 16 - (data.len() % 16);
    data.extend(vec![padding_len as u8; padding_len]);

    let mut buffer = data.clone();

    // Encrypt
    let encryptor = Aes256CbcEnc::new(&key.into(), &iv.into());
    let encrypted = encryptor
        .encrypt_padded_mut::<cbc::cipher::block_padding::NoPadding>(&mut buffer, data.len())
        .map_err(|e| format!("AES encryption failed: {}", e))?;

    use base64::Engine;
    Ok(Value::String(
        base64::engine::general_purpose::STANDARD.encode(encrypted),
    ))
}

pub fn aes_decrypt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("AES_DECRYPT expects 3 arguments (encrypted_text, key, iv)".to_string());
    }

    let encrypted = match &args[0] {
        Value::String(s) => s,
        _ => return Err("AES_DECRYPT expects a string as first argument".to_string()),
    };

    let key_str = match &args[1] {
        Value::String(s) => s,
        _ => return Err("AES_DECRYPT expects a string key as second argument".to_string()),
    };

    let iv_str = match &args[2] {
        Value::String(s) => s,
        _ => return Err("AES_DECRYPT expects a string IV as third argument".to_string()),
    };

    if key_str.len() != 32 {
        return Err("AES_DECRYPT key must be 32 characters (256 bits)".to_string());
    }
    if iv_str.len() != 16 {
        return Err("AES_DECRYPT IV must be 16 characters (128 bits)".to_string());
    }

    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];
    key.copy_from_slice(key_str.as_bytes());
    iv.copy_from_slice(iv_str.as_bytes());

    use base64::Engine;
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Invalid base64 input: {}", e))?;

    let mut buffer = ciphertext.clone();

    // Decrypt
    let decryptor = Aes256CbcDec::new(&key.into(), &iv.into());
    let plaintext = decryptor
        .decrypt_padded_mut::<cbc::cipher::block_padding::NoPadding>(&mut buffer)
        .map_err(|e| format!("AES decryption failed: {}", e))?;

    // Remove padding
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
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}
