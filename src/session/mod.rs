//! Advanced Session Management for Supabase
//!
//! This module provides comprehensive session management functionality including:
//! - Cross-tab session synchronization
//! - Platform-aware session storage (localStorage/IndexedDB/filesystem)
//! - Session encryption and secure storage
//! - Real-time session monitoring and events
//! - Offline session caching
//! - Session state persistence

pub mod encryption;
pub mod storage;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(feature = "session-management")]
use crate::auth::Session;
#[cfg(feature = "session-management")]
use crate::error::{Error, Result};
#[cfg(feature = "session-management")]
use chrono::{DateTime, Utc};
#[cfg(feature = "session-management")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "session-management")]
use std::collections::HashMap;
#[cfg(feature = "session-management")]
use std::sync::Arc;
#[cfg(feature = "session-management")]
use uuid::Uuid;

// Import StorageBackend for enum-based storage

#[cfg(all(feature = "session-management", feature = "parking_lot"))]
use parking_lot::{Mutex, RwLock};
#[cfg(all(feature = "session-management", not(feature = "parking_lot")))]
use std::sync::{Mutex, RwLock};

// Import storage backend enum
#[cfg(feature = "session-management")]
use storage::StorageBackend;

/// Session storage backend trait for cross-platform implementation
#[cfg(feature = "session-management")]
#[async_trait::async_trait]
pub trait SessionStorage: Send + Sync {
    /// Store a session with optional expiry
    async fn store_session(
        &self,
        key: &str,
        session: &SessionData,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()>;

    /// Retrieve a session by key
    async fn get_session(&self, key: &str) -> Result<Option<SessionData>>;

    /// Remove a session by key
    async fn remove_session(&self, key: &str) -> Result<()>;

    /// Clear all sessions
    async fn clear_all_sessions(&self) -> Result<()>;

    /// List all session keys
    async fn list_session_keys(&self) -> Result<Vec<String>>;

    /// Check if storage is available
    fn is_available(&self) -> bool;
}

/// Enhanced session data with metadata
#[cfg(feature = "session-management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Core session information
    pub session: Session,

    /// Session metadata
    pub metadata: SessionMetadata,

    /// Platform-specific data
    pub platform_data: HashMap<String, serde_json::Value>,
}

/// Session metadata for tracking and analytics
#[cfg(feature = "session-management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Unique session identifier
    pub session_id: Uuid,

    /// Device identifier
    pub device_id: Option<String>,

    /// Browser/client identifier
    pub client_id: Option<String>,

    /// Session creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last access timestamp
    pub last_accessed_at: DateTime<Utc>,

    /// Last refresh timestamp
    pub last_refreshed_at: Option<DateTime<Utc>>,

    /// Session source (web, mobile, desktop, etc.)
    pub source: SessionSource,

    /// IP address (if available)
    pub ip_address: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Geographic location info
    pub location: Option<SessionLocation>,

    /// Session tags for organization
    pub tags: Vec<String>,

    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

/// Session source enumeration
#[cfg(feature = "session-management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionSource {
    /// Web browser session
    Web { tab_id: Option<String> },
    /// Mobile app session
    Mobile { app_version: Option<String> },
    /// Desktop app session
    Desktop { app_version: Option<String> },
    /// Server-side session
    Server { service: Option<String> },
    /// CLI tool session
    Cli { tool_name: Option<String> },
    /// Other/unknown session source
    Other { description: String },
}

/// Geographic location information
#[cfg(feature = "session-management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLocation {
    pub country: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub timezone: Option<String>,
    pub coordinates: Option<(f64, f64)>, // (latitude, longitude)
}

/// Session event types for monitoring
#[cfg(feature = "session-management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    /// Session created
    Created { session_id: Uuid },
    /// Session updated
    Updated {
        session_id: Uuid,
        changes: Vec<String>,
    },
    /// Session accessed
    Accessed {
        session_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    /// Session refreshed
    Refreshed {
        session_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    /// Session expired
    Expired {
        session_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    /// Session destroyed
    Destroyed { session_id: Uuid, reason: String },
    /// Cross-tab sync event
    CrossTabSync {
        session_id: Uuid,
        source_tab: String,
    },
    /// Session conflict detected
    Conflict {
        session_id: Uuid,
        conflict_type: String,
    },
}

/// Cross-tab synchronization message
#[cfg(feature = "session-management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossTabMessage {
    pub message_id: Uuid,
    pub session_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub source_tab: String,
}

/// Session manager configuration
#[cfg(feature = "session-management")]
#[derive(Debug, Clone)]
pub struct SessionManagerConfig {
    /// Storage backend to use
    pub storage_backend: Arc<StorageBackend>,

    /// Enable cross-tab synchronization
    pub enable_cross_tab_sync: bool,

    /// Session key prefix for namespacing
    pub session_key_prefix: String,

    /// Default session expiry (in seconds)
    pub default_expiry_seconds: i64,

    /// Enable session encryption
    pub enable_encryption: bool,

    /// Encryption key (32 bytes)
    pub encryption_key: Option<[u8; 32]>,

    /// Enable session monitoring
    pub enable_monitoring: bool,

    /// Max number of sessions to keep in memory
    pub max_memory_sessions: usize,

    /// Background sync interval (in seconds)
    pub sync_interval_seconds: u64,
}

/// Advanced Session Manager with cross-platform support
#[cfg(feature = "session-management")]
pub struct SessionManager {
    config: SessionManagerConfig,
    active_sessions: Arc<RwLock<HashMap<Uuid, SessionData>>>,
    event_listeners: Arc<RwLock<HashMap<Uuid, SessionEventCallback>>>,
    cross_tab_channel: Arc<Mutex<Option<Box<dyn CrossTabChannel>>>>,
}

/// Session event callback type
#[cfg(feature = "session-management")]
pub type SessionEventCallback = Box<dyn Fn(SessionEvent) + Send + Sync + 'static>;

/// Cross-tab communication channel
#[cfg(feature = "session-management")]
#[async_trait::async_trait]
pub trait CrossTabChannel: Send + Sync {
    /// Send a message to other tabs
    async fn send_message(&self, message: CrossTabMessage) -> Result<()>;

    /// Register a message listener
    fn on_message(&self, callback: Box<dyn Fn(CrossTabMessage) + Send + Sync>);

    /// Close the channel
    async fn close(&self) -> Result<()>;
}

#[cfg(feature = "session-management")]
impl SessionManager {
    /// Create a new session manager
    pub fn new(config: SessionManagerConfig) -> Self {
        Self {
            config,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            event_listeners: Arc::new(RwLock::new(HashMap::new())),
            cross_tab_channel: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize the session manager
    pub async fn initialize(&self) -> Result<()> {
        // Load persisted sessions
        self.load_persisted_sessions().await?;

        // Setup cross-tab sync if enabled
        if self.config.enable_cross_tab_sync {
            self.setup_cross_tab_sync().await?;
        }

        // Start background tasks
        self.start_background_tasks().await?;

        Ok(())
    }

    /// Store a session with advanced metadata
    pub async fn store_session(&self, session: Session) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();

        let metadata = SessionMetadata {
            session_id,
            device_id: self.detect_device_id(),
            client_id: self.detect_client_id(),
            created_at: now,
            last_accessed_at: now,
            last_refreshed_at: None,
            source: self.detect_session_source(),
            ip_address: None, // TODO: Implement IP detection
            user_agent: self.detect_user_agent(),
            location: None, // TODO: Implement location detection
            tags: Vec::new(),
            custom: HashMap::new(),
        };

        let session_data = SessionData {
            session,
            metadata,
            platform_data: HashMap::new(),
        };

        // Store in memory
        {
            let mut sessions = self.active_sessions.write();
            sessions.insert(session_id, session_data.clone());
        }

        // Persist to storage
        let key = format!("{}{}", self.config.session_key_prefix, session_id);
        let expires_at = Some(session_data.session.expires_at);
        self.config
            .storage_backend
            .store_session(&key, &session_data, expires_at)
            .await?;

        // Emit event
        self.emit_session_event(SessionEvent::Created { session_id });

        // Cross-tab sync
        if self.config.enable_cross_tab_sync {
            self.sync_to_other_tabs(session_id, "session_created")
                .await?;
        }

        Ok(session_id)
    }

    /// Retrieve a session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Result<Option<SessionData>> {
        // Check memory first
        {
            let sessions = self.active_sessions.read();
            if let Some(session_data) = sessions.get(&session_id) {
                // Update last accessed time
                let mut updated_data = session_data.clone();
                updated_data.metadata.last_accessed_at = Utc::now();

                // Update in memory
                drop(sessions);
                let mut sessions = self.active_sessions.write();
                sessions.insert(session_id, updated_data.clone());

                // Emit access event
                self.emit_session_event(SessionEvent::Accessed {
                    session_id,
                    timestamp: Utc::now(),
                });

                return Ok(Some(updated_data));
            }
        }

        // Try storage if not in memory
        let key = format!("{}{}", self.config.session_key_prefix, session_id);
        if let Some(mut session_data) = self.config.storage_backend.get_session(&key).await? {
            // Update access time
            session_data.metadata.last_accessed_at = Utc::now();

            // Store in memory
            {
                let mut sessions = self.active_sessions.write();
                sessions.insert(session_id, session_data.clone());
            }

            // Emit access event
            self.emit_session_event(SessionEvent::Accessed {
                session_id,
                timestamp: Utc::now(),
            });

            Ok(Some(session_data))
        } else {
            Ok(None)
        }
    }

    /// Update a session
    pub async fn update_session(&self, session_id: Uuid, updated_session: Session) -> Result<()> {
        let mut changes = Vec::new();

        // Get current session
        if let Some(mut session_data) = self.get_session(session_id).await? {
            // Track changes
            if session_data.session.access_token != updated_session.access_token {
                changes.push("access_token".to_string());
            }
            if session_data.session.refresh_token != updated_session.refresh_token {
                changes.push("refresh_token".to_string());
            }
            if session_data.session.expires_at != updated_session.expires_at {
                changes.push("expires_at".to_string());
            }

            // Update session
            session_data.session = updated_session;
            session_data.metadata.last_accessed_at = Utc::now();

            if changes.contains(&"access_token".to_string())
                || changes.contains(&"refresh_token".to_string())
            {
                session_data.metadata.last_refreshed_at = Some(Utc::now());
            }

            // Store in memory
            {
                let mut sessions = self.active_sessions.write();
                sessions.insert(session_id, session_data.clone());
            }

            // Persist to storage
            let key = format!("{}{}", self.config.session_key_prefix, session_id);
            let expires_at = Some(session_data.session.expires_at);
            self.config
                .storage_backend
                .store_session(&key, &session_data, expires_at)
                .await?;

            // Emit event
            self.emit_session_event(SessionEvent::Updated {
                session_id,
                changes,
            });

            // Cross-tab sync
            if self.config.enable_cross_tab_sync {
                self.sync_to_other_tabs(session_id, "session_updated")
                    .await?;
            }
        } else {
            return Err(Error::auth(format!("Session {} not found", session_id)));
        }

        Ok(())
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: Uuid, reason: String) -> Result<()> {
        // Remove from memory
        {
            let mut sessions = self.active_sessions.write();
            sessions.remove(&session_id);
        }

        // Remove from storage
        let key = format!("{}{}", self.config.session_key_prefix, session_id);
        self.config.storage_backend.remove_session(&key).await?;

        // Emit event
        self.emit_session_event(SessionEvent::Destroyed { session_id, reason });

        // Cross-tab sync
        if self.config.enable_cross_tab_sync {
            self.sync_to_other_tabs(session_id, "session_destroyed")
                .await?;
        }

        Ok(())
    }

    /// List all active sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionData>> {
        let sessions = self.active_sessions.read();
        Ok(sessions.values().cloned().collect())
    }

    /// Add session event listener
    pub fn on_session_event<F>(&self, callback: F) -> Uuid
    where
        F: Fn(SessionEvent) + Send + Sync + 'static,
    {
        let listener_id = Uuid::new_v4();
        let mut listeners = self.event_listeners.write();
        listeners.insert(listener_id, Box::new(callback));
        listener_id
    }

    /// Remove session event listener
    pub fn remove_event_listener(&self, listener_id: Uuid) {
        let mut listeners = self.event_listeners.write();
        listeners.remove(&listener_id);
    }

    /// Private helper methods
    async fn load_persisted_sessions(&self) -> Result<()> {
        let keys = self.config.storage_backend.list_session_keys().await?;
        let mut valid_sessions = Vec::new();
        let mut expired_keys = Vec::new();

        // Collect valid sessions and expired keys without holding lock
        for key in keys {
            if let Some(session_data) = self.config.storage_backend.get_session(&key).await? {
                if session_data.session.expires_at > Utc::now() {
                    if let Ok(uuid) = key
                        .strip_prefix(&self.config.session_key_prefix)
                        .unwrap_or(&key)
                        .parse::<Uuid>()
                    {
                        valid_sessions.push((uuid, session_data));
                    }
                } else {
                    expired_keys.push(key);
                }
            }
        }

        // Insert valid sessions (acquire lock once)
        {
            let mut sessions = self.active_sessions.write();
            for (uuid, session_data) in valid_sessions {
                sessions.insert(uuid, session_data);
            }
        }

        // Remove expired sessions
        for key in expired_keys {
            let _ = self.config.storage_backend.remove_session(&key).await;
        }

        Ok(())
    }

    async fn setup_cross_tab_sync(&self) -> Result<()> {
        // Platform-specific cross-tab channel setup
        #[cfg(target_arch = "wasm32")]
        {
            let channel = crate::session::wasm::WasmCrossTabChannel::new()?;
            let mut cross_tab = self.cross_tab_channel.lock();
            *cross_tab = Some(Box::new(channel));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let channel = crate::session::native::NativeCrossTabChannel::new()?;
            let mut cross_tab = self.cross_tab_channel.lock();
            *cross_tab = Some(Box::new(channel));
        }

        Ok(())
    }

    async fn start_background_tasks(&self) -> Result<()> {
        // Start session cleanup task
        // Start sync task
        // Start monitoring task

        // TODO: Implement background tasks with tokio or wasm timers

        Ok(())
    }

    #[allow(clippy::await_holding_lock)]
    async fn sync_to_other_tabs(&self, session_id: Uuid, event_type: &str) -> Result<()> {
        if let Some(channel) = self.cross_tab_channel.lock().as_ref() {
            let message = CrossTabMessage {
                message_id: Uuid::new_v4(),
                session_id,
                event_type: event_type.to_string(),
                payload: serde_json::json!({}),
                timestamp: Utc::now(),
                source_tab: self
                    .detect_tab_id()
                    .unwrap_or_else(|| "unknown".to_string()),
            };

            channel.send_message(message).await?;
        }

        Ok(())
    }

    fn emit_session_event(&self, event: SessionEvent) {
        let listeners = self.event_listeners.read();
        for callback in listeners.values() {
            callback(event.clone());
        }
    }

    fn detect_device_id(&self) -> Option<String> {
        // TODO: Implement device ID detection
        None
    }

    fn detect_client_id(&self) -> Option<String> {
        // TODO: Implement client ID detection
        None
    }

    fn detect_tab_id(&self) -> Option<String> {
        // TODO: Implement tab ID detection
        None
    }

    fn detect_session_source(&self) -> SessionSource {
        #[cfg(target_arch = "wasm32")]
        {
            SessionSource::Web {
                tab_id: self.detect_tab_id(),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            SessionSource::Desktop { app_version: None }
        }
    }

    fn detect_user_agent(&self) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window().and_then(|w| w.navigator().user_agent().ok())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None
        }
    }
}

#[cfg(feature = "session-management")]
impl Default for SessionManagerConfig {
    fn default() -> Self {
        Self {
            storage_backend: Arc::new(StorageBackend::Memory(
                crate::session::storage::MemoryStorage::new(),
            )),
            enable_cross_tab_sync: true,
            session_key_prefix: "supabase_session_".to_string(),
            default_expiry_seconds: 3600, // 1 hour
            enable_encryption: false,
            encryption_key: None,
            enable_monitoring: true,
            max_memory_sessions: 100,
            sync_interval_seconds: 30,
        }
    }
}
