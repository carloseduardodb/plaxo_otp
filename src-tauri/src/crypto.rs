use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use sha2::{Sha256, Digest};

use crate::types::{AppError, Result};

const SALT: &[u8] = b"plaxo-otp-salt-2024";
const NONCE_SIZE: usize = 12;

pub fn derive_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(SALT);
    hasher.finalize().into()
}

#[allow(deprecated)]
pub fn encrypt_data(data: &str, key: &[u8; 32]) -> Result<String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    
    // Generate random nonce for each encryption
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|_| AppError::Encryption("Failed to encrypt data".to_string()))?;
    
    // Combine nonce + ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(general_purpose::STANDARD.encode(result))
}

#[allow(deprecated)]
pub fn decrypt_data(encrypted_data: &str, key: &[u8; 32]) -> Result<String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    
    let combined = general_purpose::STANDARD
        .decode(encrypted_data)
        .map_err(|_| AppError::Encryption("Invalid base64 data".to_string()))?;
    
    if combined.len() < NONCE_SIZE {
        return Err(AppError::Encryption("Data too short".to_string()));
    }
    
    // Separate nonce + ciphertext
    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError::InvalidMasterPassword)?;
    
    String::from_utf8(plaintext)
        .map_err(|_| AppError::Encryption("Invalid UTF-8 data".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let password = "test_password";
        let key = derive_key(password);
        let data = "test data";
        
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_different_keys() {
        let key1 = derive_key("password1");
        let key2 = derive_key("password2");
        let data = "test data";
        
        let encrypted = encrypt_data(data, &key1).unwrap();
        let result = decrypt_data(&encrypted, &key2);
        
        assert!(result.is_err());
    }
}
