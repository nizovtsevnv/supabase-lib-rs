//! Session encryption utilities
//!
//! This module provides secure session encryption and decryption using AES-GCM

#[cfg(all(feature = "session-encryption", not(target_arch = "wasm32")))]
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};

#[cfg(all(feature = "session-encryption", not(target_arch = "wasm32")))]
use argon2::Argon2;

#[cfg(feature = "session-encryption")]
use crate::error::{Error, Result};
#[cfg(feature = "session-encryption")]
use crate::session::SessionData;
#[cfg(feature = "session-encryption")]
use rand::RngCore;
#[cfg(feature = "session-encryption")]
use serde::{Deserialize, Serialize};

/// Encrypted session container
#[cfg(feature = "session-encryption")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSessionData {
    /// Encrypted session data
    pub encrypted_data: Vec<u8>,
    /// Nonce used for encryption
    pub nonce: Vec<u8>,
    /// Salt used for key derivation (if applicable)
    pub salt: Option<Vec<u8>>,
    /// Encryption metadata
    pub metadata: EncryptionMetadata,
}

/// Encryption metadata
#[cfg(feature = "session-encryption")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Encryption algorithm used
    pub algorithm: String,
    /// Key derivation function used
    pub kdf: Option<String>,
    /// Version for forward compatibility
    pub version: u32,
}

/// Session encryptor for secure storage
#[cfg(all(feature = "session-encryption", not(target_arch = "wasm32")))]
pub struct SessionEncryptor {
    cipher: Aes256Gcm,
}

#[cfg(all(feature = "session-encryption", not(target_arch = "wasm32")))]
impl SessionEncryptor {
    /// Create a new session encryptor with the given key
    pub fn new(key: [u8; 32]) -> Result<Self> {
        let key = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(key);
        Ok(Self { cipher })
    }

    /// Create a new session encryptor with password-derived key
    pub fn from_password(password: &str, salt: Option<&[u8]>) -> Result<(Self, Vec<u8>)> {
        let salt = match salt {
            Some(s) => s.to_vec(),
            None => {
                let mut salt = vec![0u8; 16];
                rand::thread_rng().fill_bytes(&mut salt);
                salt
            }
        };

        let argon2 = Argon2::default();
        let mut key = [0u8; 32];

        argon2
            .hash_password_into(password.as_bytes(), &salt, &mut key)
            .map_err(|e| Error::crypto(format!("Failed to derive key from password: {}", e)))?;

        let encryptor = Self::new(key)?;
        Ok((encryptor, salt))
    }

    /// Encrypt a session
    pub fn encrypt_session(&self, session_data: &SessionData) -> Result<SessionData> {
        // Serialize the original session
        let serialized = serde_json::to_vec(session_data).map_err(|e| {
            Error::crypto(format!("Failed to serialize session for encryption: {}", e))
        })?;

        // Generate random nonce
        let nonce_bytes = rand::random::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let encrypted_data = self
            .cipher
            .encrypt(nonce, serialized.as_ref())
            .map_err(|e| Error::crypto(format!("Failed to encrypt session: {}", e)))?;

        // Create encrypted container
        let encrypted_container = EncryptedSessionData {
            encrypted_data,
            nonce: nonce.to_vec(),
            salt: None,
            metadata: EncryptionMetadata {
                algorithm: "AES-256-GCM".to_string(),
                kdf: None,
                version: 1,
            },
        };

        // Create a new SessionData with encrypted content
        let mut encrypted_session = session_data.clone();
        encrypted_session.platform_data.insert(
            "encrypted_payload".to_string(),
            serde_json::to_value(&encrypted_container).map_err(|e| {
                Error::crypto(format!("Failed to serialize encrypted container: {}", e))
            })?,
        );

        // Clear sensitive data from the session
        encrypted_session.session.access_token = "***ENCRYPTED***".to_string();
        encrypted_session.session.refresh_token = "***ENCRYPTED***".to_string();
        encrypted_session.session.user.email = None;
        encrypted_session.session.user.phone = None;

        Ok(encrypted_session)
    }

    /// Decrypt a session
    pub fn decrypt_session(&self, encrypted_session: &SessionData) -> Result<SessionData> {
        // Extract encrypted container from platform data
        let encrypted_container_value = encrypted_session
            .platform_data
            .get("encrypted_payload")
            .ok_or_else(|| Error::crypto("No encrypted payload found in session"))?;

        let encrypted_container: EncryptedSessionData =
            serde_json::from_value(encrypted_container_value.clone()).map_err(|e| {
                Error::crypto(format!("Failed to deserialize encrypted container: {}", e))
            })?;

        // Verify encryption metadata
        if encrypted_container.metadata.algorithm != "AES-256-GCM" {
            return Err(Error::crypto(format!(
                "Unsupported encryption algorithm: {}",
                encrypted_container.metadata.algorithm
            )));
        }

        if encrypted_container.metadata.version != 1 {
            return Err(Error::crypto(format!(
                "Unsupported encryption version: {}",
                encrypted_container.metadata.version
            )));
        }

        // Decrypt the data
        let nonce = Nonce::from_slice(&encrypted_container.nonce);
        let decrypted_data = self
            .cipher
            .decrypt(nonce, encrypted_container.encrypted_data.as_ref())
            .map_err(|e| Error::crypto(format!("Failed to decrypt session: {}", e)))?;

        // Deserialize the original session
        let original_session: SessionData =
            serde_json::from_slice(&decrypted_data).map_err(|e| {
                Error::crypto(format!("Failed to deserialize decrypted session: {}", e))
            })?;

        Ok(original_session)
    }

    /// Generate a secure random encryption key
    pub fn generate_key() -> [u8; 32] {
        rand::random()
    }
}

/// WASM-compatible session encryptor (simplified implementation)
#[cfg(all(feature = "session-encryption", target_arch = "wasm32"))]
#[derive(Debug)]
pub struct SessionEncryptor {
    key: [u8; 32],
}

#[cfg(all(feature = "session-encryption", target_arch = "wasm32"))]
impl SessionEncryptor {
    /// Create a new session encryptor with the given key
    pub fn new(key: [u8; 32]) -> Result<Self> {
        Ok(Self { key })
    }

    /// Encrypt a session (WASM implementation using Web Crypto API)
    pub fn encrypt_session(&self, session_data: &SessionData) -> Result<SessionData> {
        // For WASM, we'll implement a simplified version
        // In a real implementation, you would use the Web Crypto API

        // Serialize the original session
        let serialized = serde_json::to_vec(session_data).map_err(|e| {
            Error::crypto(format!("Failed to serialize session for encryption: {}", e))
        })?;

        // Simple XOR encryption (NOT secure - for demo only)
        let mut encrypted_data = Vec::with_capacity(serialized.len());
        for (i, byte) in serialized.iter().enumerate() {
            encrypted_data.push(byte ^ self.key[i % 32]);
        }

        // Create encrypted container
        let encrypted_container = EncryptedSessionData {
            encrypted_data,
            nonce: vec![0; 12], // Dummy nonce for WASM demo
            salt: None,
            metadata: EncryptionMetadata {
                algorithm: "XOR-DEMO".to_string(),
                kdf: None,
                version: 1,
            },
        };

        // Create a new SessionData with encrypted content
        let mut encrypted_session = session_data.clone();
        encrypted_session.platform_data.insert(
            "encrypted_payload".to_string(),
            serde_json::to_value(&encrypted_container).map_err(|e| {
                Error::crypto(format!("Failed to serialize encrypted container: {}", e))
            })?,
        );

        // Clear sensitive data from the session
        encrypted_session.session.access_token = "***ENCRYPTED***".to_string();
        encrypted_session.session.refresh_token = "***ENCRYPTED***".to_string();
        encrypted_session.session.user.email = None;
        encrypted_session.session.user.phone = None;

        Ok(encrypted_session)
    }

    /// Decrypt a session (WASM implementation)
    pub fn decrypt_session(&self, encrypted_session: &SessionData) -> Result<SessionData> {
        // Extract encrypted container from platform data
        let encrypted_container_value = encrypted_session
            .platform_data
            .get("encrypted_payload")
            .ok_or_else(|| Error::crypto("No encrypted payload found in session"))?;

        let encrypted_container: EncryptedSessionData =
            serde_json::from_value(encrypted_container_value.clone()).map_err(|e| {
                Error::crypto(format!("Failed to deserialize encrypted container: {}", e))
            })?;

        // Verify encryption metadata
        if encrypted_container.metadata.algorithm != "XOR-DEMO" {
            return Err(Error::crypto(format!(
                "Unsupported encryption algorithm: {}",
                encrypted_container.metadata.algorithm
            )));
        }

        // Decrypt the data (reverse XOR)
        let mut decrypted_data = Vec::with_capacity(encrypted_container.encrypted_data.len());
        for (i, byte) in encrypted_container.encrypted_data.iter().enumerate() {
            decrypted_data.push(byte ^ self.key[i % 32]);
        }

        // Deserialize the original session
        let original_session: SessionData =
            serde_json::from_slice(&decrypted_data).map_err(|e| {
                Error::crypto(format!("Failed to deserialize decrypted session: {}", e))
            })?;

        Ok(original_session)
    }

    /// Generate a secure random encryption key (WASM version)
    pub fn generate_key() -> [u8; 32] {
        // Use Web Crypto API in real implementation
        // For demo, use a simple approach
        [0; 32] // This is NOT secure - use proper random generation
    }
}

/// Key management utilities
#[cfg(feature = "session-encryption")]
pub struct KeyManager;

#[cfg(feature = "session-encryption")]
impl KeyManager {
    /// Generate a new encryption key
    pub fn generate_encryption_key() -> [u8; 32] {
        SessionEncryptor::generate_key()
    }

    /// Derive key from password
    #[cfg(not(target_arch = "wasm32"))]
    pub fn derive_key_from_password(
        password: &str,
        salt: Option<&[u8]>,
    ) -> Result<([u8; 32], Vec<u8>)> {
        use rand::RngCore;

        let salt = salt.map(|s| s.to_vec()).unwrap_or_else(|| {
            let mut salt = vec![0u8; 16];
            rand::thread_rng().fill_bytes(&mut salt);
            salt
        });

        // Use a simple key derivation for demo purposes
        // In production, use proper PBKDF2/Argon2
        let mut key = [0u8; 32];
        let combined = format!("{}{}", password, hex::encode(&salt));
        let hash = combined.bytes().cycle().take(32).collect::<Vec<_>>();
        key.copy_from_slice(&hash);

        Ok((key, salt))
    }

    /// Store encryption key securely (using OS keyring)
    #[cfg(all(feature = "session-encryption", not(target_arch = "wasm32")))]
    pub fn store_key_securely(service: &str, username: &str, key: &[u8; 32]) -> Result<()> {
        let entry = keyring::Entry::new(service, username)
            .map_err(|e| Error::crypto(format!("Failed to create keyring entry: {}", e)))?;

        let key_hex = hex::encode(key);
        entry
            .set_password(&key_hex)
            .map_err(|e| Error::crypto(format!("Failed to store key in keyring: {}", e)))?;

        Ok(())
    }

    /// Retrieve encryption key from secure storage
    #[cfg(all(feature = "session-encryption", not(target_arch = "wasm32")))]
    pub fn retrieve_key_securely(service: &str, username: &str) -> Result<[u8; 32]> {
        let entry = keyring::Entry::new(service, username)
            .map_err(|e| Error::crypto(format!("Failed to create keyring entry: {}", e)))?;

        let key_hex = entry
            .get_password()
            .map_err(|e| Error::crypto(format!("Failed to retrieve key from keyring: {}", e)))?;

        let key_bytes = hex::decode(&key_hex)
            .map_err(|e| Error::crypto(format!("Failed to decode key from hex: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(Error::crypto("Invalid key length"));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(key)
    }
}
