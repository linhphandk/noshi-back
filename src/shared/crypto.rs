use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use tracing::error;

pub fn encrypt(key: &[u8], plaintext: &str) -> Result<Vec<u8>, Error> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let mut ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes()).map_err(|e| {
        error!("Encryption failed: {:?}", e);
        Error::Encryption
    })?;
    let mut out = nonce.to_vec();
    out.append(&mut ciphertext);
    Ok(out)
}

pub fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<String, Error> {
    if ciphertext.len() < 12 {
        return Err(Error::Decryption);
    }
    let (nonce_bytes, ct) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(key.into());
    let plaintext = cipher.decrypt(nonce, ct).map_err(|e| {
        error!("Decryption failed: {:?}", e);
        Error::Decryption
    })?;
    String::from_utf8(plaintext).map_err(|_| Error::Decryption)
}

pub fn decode_key(b64: &str) -> Result<[u8; 32], Error> {
    let decoded = general_purpose::STANDARD
        .decode(b64)
        .map_err(|_| Error::InvalidKey)?;
    if decoded.len() != 32 {
        return Err(Error::InvalidKey);
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&decoded);
    Ok(arr)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Encryption failed")]
    Encryption,
    #[error("Decryption failed")]
    Decryption,
    #[error("Invalid key: must be 32 bytes base64-encoded")]
    InvalidKey,
}
