//! Session storage backends
//!
//! This module provides different storage backends for session persistence:
//! - MemoryStorage: In-memory storage for testing and temporary sessions
//! - LocalStorage: Browser localStorage backend for WASM
//! - FileSystemStorage: Filesystem backend for native applications
//! - EncryptedStorage: Wrapper for encrypted storage

// Type alias for complex storage type
#[cfg(feature = "session-management")]
type SessionEntry = (SessionData, Option<DateTime<Utc>>);

#[cfg(feature = "session-management")]
use crate::error::{Error, Result};
#[cfg(feature = "session-management")]
use crate::session::{SessionData, SessionStorage};
#[cfg(feature = "session-management")]
use chrono::{DateTime, Utc};
#[cfg(feature = "session-management")]
use std::collections::HashMap;
#[cfg(feature = "session-management")]
use std::sync::{Arc, RwLock};

/// In-memory session storage (not persistent across restarts)
#[cfg(feature = "session-management")]
#[derive(Debug)]
pub struct MemoryStorage {
    sessions: Arc<RwLock<HashMap<String, SessionEntry>>>,
}

#[cfg(feature = "session-management")]
impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&self) {
        let now = Utc::now();
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, (_, expires_at)| {
            match expires_at {
                Some(expiry) => *expiry > now,
                None => true, // Keep sessions without expiry
            }
        });
    }
}

#[cfg(feature = "session-management")]
#[async_trait::async_trait]
impl SessionStorage for MemoryStorage {
    async fn store_session(
        &self,
        key: &str,
        session: &SessionData,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| Error::storage("Failed to acquire write lock for memory storage"))?;
        sessions.insert(key.to_string(), (session.clone(), expires_at));
        Ok(())
    }

    async fn get_session(&self, key: &str) -> Result<Option<SessionData>> {
        let sessions = self
            .sessions
            .read()
            .map_err(|_| Error::storage("Failed to acquire read lock for memory storage"))?;

        if let Some((session_data, expires_at)) = sessions.get(key) {
            // Check if session is expired
            if let Some(expiry) = expires_at {
                if *expiry <= Utc::now() {
                    return Ok(None);
                }
            }
            Ok(Some(session_data.clone()))
        } else {
            Ok(None)
        }
    }

    async fn remove_session(&self, key: &str) -> Result<()> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| Error::storage("Failed to acquire write lock for memory storage"))?;
        sessions.remove(key);
        Ok(())
    }

    async fn clear_all_sessions(&self) -> Result<()> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| Error::storage("Failed to acquire write lock for memory storage"))?;
        sessions.clear();
        Ok(())
    }

    async fn list_session_keys(&self) -> Result<Vec<String>> {
        let sessions = self
            .sessions
            .read()
            .map_err(|_| Error::storage("Failed to acquire read lock for memory storage"))?;
        Ok(sessions.keys().cloned().collect())
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(feature = "session-management")]
impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Browser localStorage backend for WASM
#[cfg(all(feature = "session-management", target_arch = "wasm32"))]
#[derive(Debug)]
pub struct LocalStorage {
    key_prefix: String,
}

#[cfg(all(feature = "session-management", target_arch = "wasm32"))]
impl LocalStorage {
    pub fn new(key_prefix: Option<String>) -> Result<Self> {
        // Check if localStorage is available
        if !Self::is_storage_available() {
            return Err(Error::storage(
                "localStorage is not available in this environment",
            ));
        }

        Ok(Self {
            key_prefix: key_prefix.unwrap_or_else(|| "supabase_".to_string()),
        })
    }

    fn is_storage_available() -> bool {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .is_some()
    }

    fn get_storage() -> Result<web_sys::Storage> {
        web_sys::window()
            .ok_or_else(|| Error::storage("No window object available"))?
            .local_storage()
            .map_err(|_| Error::storage("Failed to access localStorage"))?
            .ok_or_else(|| Error::storage("localStorage is not available"))
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }
}

#[cfg(all(feature = "session-management", target_arch = "wasm32"))]
#[async_trait::async_trait]
impl SessionStorage for LocalStorage {
    async fn store_session(
        &self,
        key: &str,
        session: &SessionData,
        _expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let storage = Self::get_storage()?;
        let storage_key = self.make_key(key);
        let serialized = serde_json::to_string(session)
            .map_err(|e| Error::storage(format!("Failed to serialize session: {}", e)))?;

        storage
            .set_item(&storage_key, &serialized)
            .map_err(|_| Error::storage("Failed to store session in localStorage"))?;

        Ok(())
    }

    async fn get_session(&self, key: &str) -> Result<Option<SessionData>> {
        let storage = Self::get_storage()?;
        let storage_key = self.make_key(key);

        match storage.get_item(&storage_key) {
            Ok(Some(serialized)) => {
                let session_data: SessionData = serde_json::from_str(&serialized)
                    .map_err(|e| Error::storage(format!("Failed to deserialize session: {}", e)))?;

                // Check if session is expired
                if session_data.session.expires_at <= Utc::now() {
                    // Remove expired session
                    let _ = self.remove_session(key).await;
                    Ok(None)
                } else {
                    Ok(Some(session_data))
                }
            }
            Ok(None) => Ok(None),
            Err(_) => Err(Error::storage("Failed to read from localStorage")),
        }
    }

    async fn remove_session(&self, key: &str) -> Result<()> {
        let storage = Self::get_storage()?;
        let storage_key = self.make_key(key);
        storage
            .remove_item(&storage_key)
            .map_err(|_| Error::storage("Failed to remove session from localStorage"))?;
        Ok(())
    }

    async fn clear_all_sessions(&self) -> Result<()> {
        let storage = Self::get_storage()?;
        let keys_to_remove: Vec<String> = (0..storage.length().unwrap_or(0))
            .filter_map(|i| storage.key(i).ok().flatten())
            .filter(|key| key.starts_with(&self.key_prefix))
            .collect();

        for key in keys_to_remove {
            storage
                .remove_item(&key)
                .map_err(|_| Error::storage("Failed to clear session from localStorage"))?;
        }

        Ok(())
    }

    async fn list_session_keys(&self) -> Result<Vec<String>> {
        let storage = Self::get_storage()?;
        let keys: Vec<String> = (0..storage.length().unwrap_or(0))
            .filter_map(|i| storage.key(i).ok().flatten())
            .filter(|key| key.starts_with(&self.key_prefix))
            .map(|key| {
                key.strip_prefix(&self.key_prefix)
                    .unwrap_or(&key)
                    .to_string()
            })
            .collect();

        Ok(keys)
    }

    fn is_available(&self) -> bool {
        Self::is_storage_available()
    }
}

/// Filesystem-based storage for native applications
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
#[derive(Debug)]
pub struct FileSystemStorage {
    base_dir: std::path::PathBuf,
}

#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
impl FileSystemStorage {
    pub fn new(base_dir: Option<std::path::PathBuf>) -> Result<Self> {
        let base_dir = match base_dir {
            Some(dir) => dir,
            None => {
                // Use OS-appropriate data directory
                dirs::data_local_dir()
                    .ok_or_else(|| Error::storage("Could not determine data directory"))?
                    .join("supabase-sessions")
            }
        };

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&base_dir)
            .map_err(|e| Error::storage(format!("Failed to create session directory: {}", e)))?;

        Ok(Self { base_dir })
    }

    fn get_session_path(&self, key: &str) -> std::path::PathBuf {
        self.base_dir.join(format!("{}.json", key))
    }
}

#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
#[async_trait::async_trait]
impl SessionStorage for FileSystemStorage {
    async fn store_session(
        &self,
        key: &str,
        session: &SessionData,
        _expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let path = self.get_session_path(key);
        let serialized = serde_json::to_string_pretty(session)
            .map_err(|e| Error::storage(format!("Failed to serialize session: {}", e)))?;

        tokio::fs::write(&path, serialized)
            .await
            .map_err(|e| Error::storage(format!("Failed to write session file: {}", e)))?;

        Ok(())
    }

    async fn get_session(&self, key: &str) -> Result<Option<SessionData>> {
        let path = self.get_session_path(key);

        match tokio::fs::read_to_string(&path).await {
            Ok(serialized) => {
                let session_data: SessionData = serde_json::from_str(&serialized)
                    .map_err(|e| Error::storage(format!("Failed to deserialize session: {}", e)))?;

                // Check if session is expired
                if session_data.session.expires_at <= Utc::now() {
                    // Remove expired session
                    let _ = self.remove_session(key).await;
                    Ok(None)
                } else {
                    Ok(Some(session_data))
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::storage(format!(
                "Failed to read session file: {}",
                e
            ))),
        }
    }

    async fn remove_session(&self, key: &str) -> Result<()> {
        let path = self.get_session_path(key);
        match tokio::fs::remove_file(&path).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Already removed
            Err(e) => Err(Error::storage(format!(
                "Failed to remove session file: {}",
                e
            ))),
        }
    }

    async fn clear_all_sessions(&self) -> Result<()> {
        let mut dir_entries = tokio::fs::read_dir(&self.base_dir)
            .await
            .map_err(|e| Error::storage(format!("Failed to read session directory: {}", e)))?;

        while let Some(entry) = dir_entries
            .next_entry()
            .await
            .map_err(|e| Error::storage(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match tokio::fs::remove_file(&path).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("Failed to remove session file {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn list_session_keys(&self) -> Result<Vec<String>> {
        let mut dir_entries = tokio::fs::read_dir(&self.base_dir)
            .await
            .map_err(|e| Error::storage(format!("Failed to read session directory: {}", e)))?;

        let mut keys = Vec::new();

        while let Some(entry) = dir_entries
            .next_entry()
            .await
            .map_err(|e| Error::storage(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    keys.push(file_stem.to_string());
                }
            }
        }

        Ok(keys)
    }

    fn is_available(&self) -> bool {
        self.base_dir.exists() && self.base_dir.is_dir()
    }
}

/// Encrypted storage wrapper
#[cfg(all(feature = "session-management", feature = "session-encryption"))]
pub struct EncryptedStorage {
    inner: Arc<dyn SessionStorage>,
    encryptor: Arc<crate::session::encryption::SessionEncryptor>,
}

#[cfg(all(feature = "session-management", feature = "session-encryption"))]
impl std::fmt::Debug for EncryptedStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedStorage")
            .field("inner", &"Arc<dyn SessionStorage>")
            .field("encryptor", &"Arc<SessionEncryptor>")
            .finish()
    }
}

#[cfg(all(feature = "session-management", feature = "session-encryption"))]
impl EncryptedStorage {
    pub fn new(inner: Arc<dyn SessionStorage>, encryption_key: [u8; 32]) -> Result<Self> {
        let encryptor = Arc::new(crate::session::encryption::SessionEncryptor::new(
            encryption_key,
        )?);
        Ok(Self { inner, encryptor })
    }
}

#[cfg(all(feature = "session-management", feature = "session-encryption"))]
#[async_trait::async_trait]
impl SessionStorage for EncryptedStorage {
    async fn store_session(
        &self,
        key: &str,
        session: &SessionData,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let encrypted_session = self.encryptor.encrypt_session(session)?;
        self.inner
            .store_session(key, &encrypted_session, expires_at)
            .await
    }

    async fn get_session(&self, key: &str) -> Result<Option<SessionData>> {
        if let Some(encrypted_session) = self.inner.get_session(key).await? {
            let decrypted_session = self.encryptor.decrypt_session(&encrypted_session)?;
            Ok(Some(decrypted_session))
        } else {
            Ok(None)
        }
    }

    async fn remove_session(&self, key: &str) -> Result<()> {
        self.inner.remove_session(key).await
    }

    async fn clear_all_sessions(&self) -> Result<()> {
        self.inner.clear_all_sessions().await
    }

    async fn list_session_keys(&self) -> Result<Vec<String>> {
        self.inner.list_session_keys().await
    }

    fn is_available(&self) -> bool {
        self.inner.is_available()
    }
}

/// Enum-based storage backend for dyn compatibility
#[cfg(feature = "session-management")]
#[derive(Debug)]
pub enum StorageBackend {
    Memory(MemoryStorage),
    #[cfg(target_arch = "wasm32")]
    LocalStorage(LocalStorage),
    #[cfg(not(target_arch = "wasm32"))]
    FileSystem(FileSystemStorage),
    #[cfg(feature = "session-encryption")]
    Encrypted(EncryptedStorage),
}

#[cfg(feature = "session-management")]
impl StorageBackend {
    /// Store a session with optional expiry
    pub async fn store_session(
        &self,
        key: &str,
        session: &SessionData,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        match self {
            StorageBackend::Memory(storage) => {
                storage.store_session(key, session, expires_at).await
            }
            #[cfg(target_arch = "wasm32")]
            StorageBackend::LocalStorage(storage) => {
                storage.store_session(key, session, expires_at).await
            }
            #[cfg(not(target_arch = "wasm32"))]
            StorageBackend::FileSystem(storage) => {
                storage.store_session(key, session, expires_at).await
            }
            #[cfg(feature = "session-encryption")]
            StorageBackend::Encrypted(storage) => {
                storage.store_session(key, session, expires_at).await
            }
        }
    }

    /// Retrieve a session by key
    pub async fn get_session(&self, key: &str) -> Result<Option<SessionData>> {
        match self {
            StorageBackend::Memory(storage) => storage.get_session(key).await,
            #[cfg(target_arch = "wasm32")]
            StorageBackend::LocalStorage(storage) => storage.get_session(key).await,
            #[cfg(not(target_arch = "wasm32"))]
            StorageBackend::FileSystem(storage) => storage.get_session(key).await,
            #[cfg(feature = "session-encryption")]
            StorageBackend::Encrypted(storage) => storage.get_session(key).await,
        }
    }

    /// Remove a session by key
    pub async fn remove_session(&self, key: &str) -> Result<()> {
        match self {
            StorageBackend::Memory(storage) => storage.remove_session(key).await,
            #[cfg(target_arch = "wasm32")]
            StorageBackend::LocalStorage(storage) => storage.remove_session(key).await,
            #[cfg(not(target_arch = "wasm32"))]
            StorageBackend::FileSystem(storage) => storage.remove_session(key).await,
            #[cfg(feature = "session-encryption")]
            StorageBackend::Encrypted(storage) => storage.remove_session(key).await,
        }
    }

    /// Clear all sessions
    pub async fn clear_all_sessions(&self) -> Result<()> {
        match self {
            StorageBackend::Memory(storage) => storage.clear_all_sessions().await,
            #[cfg(target_arch = "wasm32")]
            StorageBackend::LocalStorage(storage) => storage.clear_all_sessions().await,
            #[cfg(not(target_arch = "wasm32"))]
            StorageBackend::FileSystem(storage) => storage.clear_all_sessions().await,
            #[cfg(feature = "session-encryption")]
            StorageBackend::Encrypted(storage) => storage.clear_all_sessions().await,
        }
    }

    /// List all session keys
    pub async fn list_session_keys(&self) -> Result<Vec<String>> {
        match self {
            StorageBackend::Memory(storage) => storage.list_session_keys().await,
            #[cfg(target_arch = "wasm32")]
            StorageBackend::LocalStorage(storage) => storage.list_session_keys().await,
            #[cfg(not(target_arch = "wasm32"))]
            StorageBackend::FileSystem(storage) => storage.list_session_keys().await,
            #[cfg(feature = "session-encryption")]
            StorageBackend::Encrypted(storage) => storage.list_session_keys().await,
        }
    }

    /// Check if storage is available
    pub fn is_available(&self) -> bool {
        match self {
            StorageBackend::Memory(storage) => storage.is_available(),
            #[cfg(target_arch = "wasm32")]
            StorageBackend::LocalStorage(storage) => storage.is_available(),
            #[cfg(not(target_arch = "wasm32"))]
            StorageBackend::FileSystem(storage) => storage.is_available(),
            #[cfg(feature = "session-encryption")]
            StorageBackend::Encrypted(storage) => storage.is_available(),
        }
    }
}

/// Factory function to create the appropriate storage backend
#[cfg(feature = "session-management")]
pub fn create_default_storage() -> Result<Arc<StorageBackend>> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(storage) = LocalStorage::new(None) {
            Ok(Arc::new(StorageBackend::LocalStorage(storage)))
        } else {
            // Fallback to memory storage
            Ok(Arc::new(StorageBackend::Memory(MemoryStorage::new())))
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(storage) = FileSystemStorage::new(None) {
            Ok(Arc::new(StorageBackend::FileSystem(storage)))
        } else {
            // Fallback to memory storage
            Ok(Arc::new(StorageBackend::Memory(MemoryStorage::new())))
        }
    }
}
