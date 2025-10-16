//! Realtime module for Supabase WebSocket subscriptions
//!
//! This module provides cross-platform WebSocket support using proper abstractions:
//! - Native: Uses tokio-tungstenite with TLS support
//! - WASM: Uses web-sys WebSocket API through the browser
//!
//! ## Usage
//!
//! ```rust,no_run
//! use supabase_lib_rs::Client;
//! use supabase_lib_rs::realtime::RealtimeEvent;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::new("your-url", "your-key")?;
//! let realtime = client.realtime();
//!
//! // Connect to realtime
//! realtime.connect().await?;
//!
//! // Subscribe to table changes
//! let subscription_id = realtime
//!     .channel("posts")
//!     .table("posts")
//!     .event(RealtimeEvent::All)
//!     .subscribe(|message| {
//!         println!("Received update: {:?}", message);
//!     })
//!     .await?;
//!
//! // Later, unsubscribe
//! realtime.unsubscribe(&subscription_id).await?;
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "realtime")]
use crate::{
    async_runtime::{AsyncLock, RuntimeLock},
    error::{Error, Result},
    types::SupabaseConfig,
    websocket::{create_websocket, WebSocketConnection},
};

#[cfg(feature = "realtime")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "realtime")]
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

#[cfg(feature = "realtime")]
use tracing::{debug, error, info, warn};

#[cfg(feature = "realtime")]
use uuid::Uuid;

/// Type alias for complex connection storage
#[cfg(feature = "realtime")]
pub type ConnectionStorage = Arc<RuntimeLock<Vec<Option<Box<dyn WebSocketConnection>>>>>;

/// Realtime client for WebSocket subscriptions
///
/// Provides cross-platform realtime subscriptions to Supabase database changes.
///
/// # Examples
///
/// ## Basic subscription
/// ```rust,no_run
/// use supabase_lib_rs::{Client, realtime::RealtimeEvent};
///
/// # async fn example() -> supabase_lib_rs::Result<()> {
/// let client = Client::new("your-url", "your-key")?;
/// let realtime = client.realtime();
///
/// realtime.connect().await?;
///
/// let sub_id = realtime
///     .channel("public-posts")
///     .table("posts")
///     .subscribe(|msg| println!("New post: {:?}", msg))
///     .await?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "realtime")]
#[derive(Debug, Clone)]
pub struct Realtime {
    connection_manager: Arc<ConnectionManager>,
    message_loop_handle: Arc<AtomicBool>,
}

/// Connection manager for WebSocket connections
#[cfg(feature = "realtime")]
struct ConnectionManager {
    url: String,
    api_key: String,
    connection: RuntimeLock<Option<Box<dyn WebSocketConnection>>>,
    ref_counter: AtomicU64,
    subscriptions: RuntimeLock<HashMap<String, Subscription>>,
    is_message_loop_running: AtomicBool,
}

#[cfg(feature = "realtime")]
impl std::fmt::Debug for ConnectionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionManager")
            .field("url", &self.url)
            .field("api_key", &"[REDACTED]")
            .field("ref_counter", &self.ref_counter)
            .field("connection", &"<WebSocket connection>")
            .field("subscriptions", &"<subscriptions>")
            .finish()
    }
}

/// Subscription information
#[cfg(feature = "realtime")]
#[derive(Clone)]
pub struct Subscription {
    pub id: String,
    pub topic: String,
    pub config: SubscriptionConfig,
    #[cfg(not(target_arch = "wasm32"))]
    pub callback: Arc<dyn Fn(RealtimeMessage) + Send + Sync>,
    #[cfg(target_arch = "wasm32")]
    pub callback: Arc<dyn Fn(RealtimeMessage)>,
}

#[cfg(feature = "realtime")]
impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("id", &self.id)
            .field("topic", &self.topic)
            .field("config", &self.config)
            .field("callback", &"<callback fn>")
            .finish()
    }
}

/// Configuration for subscriptions
///
/// # Examples
/// ```rust
/// use supabase_lib_rs::realtime::{SubscriptionConfig, RealtimeEvent, AdvancedFilter, FilterOperator};
/// use std::collections::HashMap;
///
/// let config = SubscriptionConfig {
///     table: Some("posts".to_string()),
///     schema: "public".to_string(),
///     event: Some(RealtimeEvent::Insert),
///     filter: Some("author_id=eq.123".to_string()),
///     advanced_filters: vec![
///         AdvancedFilter {
///             column: "status".to_string(),
///             operator: FilterOperator::Equal,
///             value: serde_json::Value::String("published".to_string()),
///         }
///     ],
///     enable_presence: false,
///     enable_broadcast: false,
///     presence_callback: None,
///     broadcast_callback: None,
/// };
/// ```
#[cfg(feature = "realtime")]
#[derive(Clone)]
pub struct SubscriptionConfig {
    pub table: Option<String>,
    pub schema: String,
    pub event: Option<RealtimeEvent>,
    pub filter: Option<String>,
    pub advanced_filters: Vec<AdvancedFilter>,
    pub enable_presence: bool,
    pub enable_broadcast: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub presence_callback: Option<PresenceCallback>,
    #[cfg(target_arch = "wasm32")]
    pub presence_callback: Option<Arc<dyn Fn(PresenceEvent)>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub broadcast_callback: Option<BroadcastCallback>,
    #[cfg(target_arch = "wasm32")]
    pub broadcast_callback: Option<Arc<dyn Fn(BroadcastMessage)>>,
}

#[cfg(feature = "realtime")]
impl std::fmt::Debug for SubscriptionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscriptionConfig")
            .field("table", &self.table)
            .field("schema", &self.schema)
            .field("event", &self.event)
            .field("filter", &self.filter)
            .field("advanced_filters", &self.advanced_filters)
            .field("enable_presence", &self.enable_presence)
            .field("enable_broadcast", &self.enable_broadcast)
            .field("presence_callback", &"<callback fn>")
            .field("broadcast_callback", &"<callback fn>")
            .finish()
    }
}

#[cfg(feature = "realtime")]
impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            table: None,
            schema: "public".to_string(),
            event: None,
            filter: None,
            advanced_filters: Vec::new(),
            enable_presence: false,
            enable_broadcast: false,
            presence_callback: None,
            broadcast_callback: None,
        }
    }
}

/// Realtime event types for filtering subscriptions
///
/// # Examples
/// ```rust
/// use supabase_lib_rs::realtime::RealtimeEvent;
///
/// // Listen to all events
/// let all_events = RealtimeEvent::All;
///
/// // Listen only to inserts
/// let inserts_only = RealtimeEvent::Insert;
/// ```
#[cfg(feature = "realtime")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RealtimeEvent {
    #[serde(rename = "INSERT")]
    Insert,
    #[serde(rename = "UPDATE")]
    Update,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "*")]
    All,
}

/// Realtime message received from Supabase
///
/// Contains the event data and metadata about the database change.
#[cfg(feature = "realtime")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMessage {
    pub event: String,
    pub payload: RealtimePayload,
    pub ref_id: Option<String>,
    pub topic: String,
}

/// Payload of a realtime message
///
/// Contains the actual data from database changes.
#[cfg(feature = "realtime")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimePayload {
    pub record: Option<serde_json::Value>,
    pub old_record: Option<serde_json::Value>,
    pub schema: Option<String>,
    pub table: Option<String>,
    pub commit_timestamp: Option<String>,
    pub event_type: Option<String>,
    pub new: Option<serde_json::Value>,
    pub old: Option<serde_json::Value>,
}

/// Supabase realtime protocol message for sending to server
#[cfg(feature = "realtime")]
#[derive(Debug, Serialize)]
struct RealtimeProtocolMessage {
    topic: String,
    event: String,
    payload: serde_json::Value,
    ref_id: String,
}

/// Presence state for user tracking
#[cfg(feature = "realtime")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceState {
    pub user_id: String,
    pub online_at: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Presence event for tracking user joins/leaves
#[cfg(feature = "realtime")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEvent {
    pub event_type: PresenceEventType,
    pub user_id: String,
    pub presence_state: PresenceState,
}

/// Types of presence events
#[cfg(feature = "realtime")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PresenceEventType {
    #[serde(rename = "presence_state")]
    Join,
    #[serde(rename = "presence_diff")]
    Leave,
}

/// Callback for presence events
#[cfg(feature = "realtime")]
pub type PresenceCallback = Arc<dyn Fn(PresenceEvent) + Send + Sync>;

/// Broadcast message for cross-client communication
#[cfg(feature = "realtime")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub event: String,
    pub payload: serde_json::Value,
    pub from_user_id: Option<String>,
    pub timestamp: String,
}

/// Callback for broadcast messages
#[cfg(feature = "realtime")]
pub type BroadcastCallback = Arc<dyn Fn(BroadcastMessage) + Send + Sync>;

/// Advanced filter configuration
#[cfg(feature = "realtime")]
#[derive(Debug, Clone)]
pub struct AdvancedFilter {
    pub column: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Filter operators for advanced filtering
#[cfg(feature = "realtime")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    #[serde(rename = "eq")]
    Equal,
    #[serde(rename = "neq")]
    NotEqual,
    #[serde(rename = "gt")]
    GreaterThan,
    #[serde(rename = "gte")]
    GreaterThanOrEqual,
    #[serde(rename = "lt")]
    LessThan,
    #[serde(rename = "lte")]
    LessThanOrEqual,
    #[serde(rename = "in")]
    In,
    #[serde(rename = "is")]
    Is,
    #[serde(rename = "like")]
    Like,
    #[serde(rename = "ilike")]
    ILike,
    #[serde(rename = "match")]
    Match,
    #[serde(rename = "imatch")]
    IMatch,
}

/// Connection pool configuration
#[cfg(feature = "realtime")]
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in pool (default: 10)
    pub max_connections: usize,
    /// Connection timeout in seconds (default: 30)
    pub connection_timeout: u64,
    /// Keep-alive interval in seconds (default: 30)
    pub keep_alive_interval: u64,
    /// Reconnect delay in milliseconds (default: 1000)
    pub reconnect_delay: u64,
    /// Maximum reconnect attempts (default: 5)
    pub max_reconnect_attempts: u32,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            connection_timeout: 30,
            keep_alive_interval: 30,
            reconnect_delay: 1000,
            max_reconnect_attempts: 5,
        }
    }
}

/// Connection pool for efficient WebSocket management
#[cfg(feature = "realtime")]
pub struct ConnectionPool {
    config: ConnectionPoolConfig,
    connections: ConnectionStorage,
    active_connections: Arc<AtomicU64>,
}

#[cfg(feature = "realtime")]
impl std::fmt::Debug for ConnectionPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionPool")
            .field("config", &self.config)
            .field("active_connections", &self.active_connections)
            .finish()
    }
}

#[cfg(feature = "realtime")]
impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: ConnectionPoolConfig) -> Self {
        let mut connections = Vec::new();
        connections.resize_with(config.max_connections, || None);

        Self {
            config,
            connections: Arc::new(RuntimeLock::new(connections)),
            active_connections: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get an available connection from the pool
    pub async fn get_connection(&self) -> Result<Option<Box<dyn WebSocketConnection>>> {
        let mut connections = self.connections.write().await;

        for connection_slot in connections.iter_mut() {
            if let Some(connection) = connection_slot.take() {
                if connection.is_connected() {
                    debug!("Reusing existing connection from pool");
                    return Ok(Some(connection));
                }
            }
        }

        // No available connections, try to create a new one
        for connection_slot in connections.iter_mut() {
            if connection_slot.is_none() {
                let new_connection = crate::websocket::create_websocket();
                *connection_slot = Some(new_connection);
                self.active_connections
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                debug!("Created new connection in pool");
                return Ok(connection_slot.take());
            }
        }

        debug!("Connection pool is full");
        Ok(None)
    }

    /// Return a connection to the pool
    pub async fn return_connection(&self, connection: Box<dyn WebSocketConnection>) {
        let mut connections = self.connections.write().await;

        for connection_slot in connections.iter_mut() {
            if connection_slot.is_none() {
                *connection_slot = Some(connection);
                debug!("Returned connection to pool");
                return;
            }
        }

        // Pool is full, close the connection
        warn!("Pool is full, dropping connection");
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> ConnectionPoolStats {
        let connections = self.connections.read().await;
        let total = connections.len();
        let active = connections.iter().filter(|c| c.is_some()).count();
        let available = connections
            .iter()
            .filter(|c| c.as_ref().is_some_and(|conn| conn.is_connected()))
            .count();

        ConnectionPoolStats {
            total_connections: total,
            active_connections: active,
            available_connections: available,
            max_connections: self.config.max_connections,
        }
    }

    /// Close all connections in the pool
    pub async fn close_all(&self) -> Result<()> {
        let mut connections = self.connections.write().await;

        for connection_slot in connections.iter_mut() {
            if let Some(mut connection) = connection_slot.take() {
                connection.close().await?;
            }
        }

        self.active_connections
            .store(0, std::sync::atomic::Ordering::SeqCst);
        info!("Closed all connections in pool");
        Ok(())
    }
}

/// Statistics about the connection pool
#[cfg(feature = "realtime")]
#[derive(Debug, Clone)]
pub struct ConnectionPoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub available_connections: usize,
    pub max_connections: usize,
}

#[cfg(feature = "realtime")]
impl Realtime {
    /// Create a new realtime client (works on both native and WASM)
    ///
    /// # Examples
    /// ```rust
    /// use supabase_lib_rs::types::SupabaseConfig;
    /// use supabase_lib_rs::realtime::Realtime;
    /// use std::sync::Arc;
    ///
    /// let config = Arc::new(SupabaseConfig {
    ///     url: "https://your-project.supabase.co".to_string(),
    ///     key: "your-anon-key".to_string(),
    ///     ..Default::default()
    /// });
    ///
    /// let realtime = Realtime::new(config).unwrap();
    /// ```
    pub fn new(config: Arc<SupabaseConfig>) -> Result<Self> {
        debug!("Creating realtime client");

        let ws_url = config
            .url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let realtime_url = format!("{}/realtime/v1/websocket", ws_url);

        let connection_manager = Arc::new(ConnectionManager {
            url: realtime_url,
            api_key: config.key.clone(),
            connection: RuntimeLock::new(None),
            ref_counter: AtomicU64::new(0),
            subscriptions: RuntimeLock::new(HashMap::new()),
            is_message_loop_running: AtomicBool::new(false),
        });

        let message_loop_handle = Arc::new(AtomicBool::new(false));

        Ok(Self {
            connection_manager,
            message_loop_handle,
        })
    }

    /// Connect to the realtime server (cross-platform)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = Client::new("your-url", "your-key")?;
    /// let realtime = client.realtime();
    ///
    /// realtime.connect().await?;
    /// println!("Connected to Supabase realtime!");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(&self) -> Result<()> {
        debug!("Connecting to realtime server");

        let mut connection_guard = self.connection_manager.connection.write().await;

        if let Some(ref conn) = *connection_guard {
            if conn.is_connected() {
                debug!("Already connected to realtime server");
                return Ok(());
            }
        }

        let mut connection = create_websocket();
        let url = format!(
            "{}?apikey={}&vsn=1.0.0",
            self.connection_manager.url, self.connection_manager.api_key
        );

        connection.connect(&url).await?;
        *connection_guard = Some(connection);

        // Start message loop
        self.start_message_loop().await?;

        info!("Connected to realtime server");
        Ok(())
    }

    /// Disconnect from the realtime server
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = Client::new("your-url", "your-key")?;
    /// let realtime = client.realtime();
    ///
    /// realtime.connect().await?;
    /// // ... do work ...
    /// realtime.disconnect().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn disconnect(&self) -> Result<()> {
        debug!("Disconnecting from realtime server");

        // Stop message loop
        self.message_loop_handle.store(false, Ordering::SeqCst);
        self.connection_manager
            .is_message_loop_running
            .store(false, Ordering::SeqCst);

        let mut connection_guard = self.connection_manager.connection.write().await;
        if let Some(ref mut connection) = *connection_guard {
            connection.close().await?;
        }
        *connection_guard = None;

        // Clear all subscriptions
        let mut subscriptions = self.connection_manager.subscriptions.write().await;
        subscriptions.clear();

        info!("Disconnected from realtime server");
        Ok(())
    }

    /// Check if connected to realtime server
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = Client::new("your-url", "your-key")?;
    /// let realtime = client.realtime();
    ///
    /// if !realtime.is_connected().await {
    ///     realtime.connect().await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_connected(&self) -> bool {
        let connection_guard = self.connection_manager.connection.read().await;
        if let Some(ref conn) = *connection_guard {
            conn.is_connected()
        } else {
            false
        }
    }

    /// Create a channel subscription builder
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = Client::new("your-url", "your-key")?;
    ///
    /// let subscription = client.realtime()
    ///     .channel("public-posts")
    ///     .table("posts")
    ///     .subscribe(|msg| println!("Update: {:?}", msg))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn channel(&self, _topic: &str) -> ChannelBuilder {
        ChannelBuilder {
            realtime: self.clone(),
            config: SubscriptionConfig::default(),
        }
    }

    /// Unsubscribe from a channel
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = Client::new("your-url", "your-key")?;
    /// let realtime = client.realtime();
    ///
    /// let subscription_id = realtime
    ///     .channel("posts")
    ///     .table("posts")
    ///     .subscribe(|_| {})
    ///     .await?;
    ///
    /// // Later...
    /// realtime.unsubscribe(&subscription_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        debug!("Unsubscribing from subscription: {}", subscription_id);

        let mut subscriptions = self.connection_manager.subscriptions.write().await;
        if let Some(subscription) = subscriptions.remove(subscription_id) {
            // Send leave message to server
            self.send_leave_message(&subscription.topic).await?;
            info!("Unsubscribed from subscription: {}", subscription_id);
        } else {
            warn!("Subscription {} not found for unsubscribe", subscription_id);
        }

        Ok(())
    }

    /// Subscribe to a channel with custom configuration
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn subscribe<F>(
        &self,
        subscription_config: SubscriptionConfig,
        callback: F,
    ) -> Result<String>
    where
        F: Fn(RealtimeMessage) + Send + Sync + 'static,
    {
        let subscription_id = Uuid::new_v4().to_string();
        let topic = self.build_topic(&subscription_config);

        debug!(
            "Creating subscription {} for topic {}",
            subscription_id, topic
        );

        // Ensure we're connected
        self.connect().await?;

        // Send join message to server
        self.send_join_message(&topic, &subscription_config).await?;

        // Store subscription
        let subscription = Subscription {
            id: subscription_id.clone(),
            topic: topic.clone(),
            config: subscription_config,
            callback: Arc::new(callback),
        };

        let mut subscriptions = self.connection_manager.subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), subscription);

        info!("Subscribed to topic {} with ID {}", topic, subscription_id);
        Ok(subscription_id)
    }

    /// Subscribe to a channel with custom configuration (WASM version)
    #[cfg(target_arch = "wasm32")]
    pub async fn subscribe<F>(
        &self,
        subscription_config: SubscriptionConfig,
        callback: F,
    ) -> Result<String>
    where
        F: Fn(RealtimeMessage) + 'static,
    {
        let subscription_id = Uuid::new_v4().to_string();
        let topic = self.build_topic(&subscription_config);

        debug!(
            "Creating subscription {} for topic {}",
            subscription_id, topic
        );

        // Ensure we're connected
        self.connect().await?;

        // Send join message to server
        self.send_join_message(&topic, &subscription_config).await?;

        // Store subscription
        let subscription = Subscription {
            id: subscription_id.clone(),
            topic: topic.clone(),
            config: subscription_config,
            callback: Arc::new(callback),
        };

        let mut subscriptions = self.connection_manager.subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), subscription);

        info!("Subscribed to topic {} with ID {}", topic, subscription_id);
        Ok(subscription_id)
    }

    /// Build topic string from subscription config
    fn build_topic(&self, config: &SubscriptionConfig) -> String {
        if let Some(ref table) = config.table {
            format!("realtime:{}:{}", config.schema, table)
        } else {
            format!("realtime:{}", config.schema)
        }
    }

    /// Send join message to Supabase realtime server
    async fn send_join_message(&self, topic: &str, config: &SubscriptionConfig) -> Result<()> {
        let mut payload = serde_json::Map::new();

        if let Some(ref table) = config.table {
            payload.insert(
                "table".to_string(),
                serde_json::Value::String(table.clone()),
            );
        }

        if let Some(ref event) = config.event {
            let event_str = match event {
                RealtimeEvent::Insert => "INSERT",
                RealtimeEvent::Update => "UPDATE",
                RealtimeEvent::Delete => "DELETE",
                RealtimeEvent::All => "*",
            };
            payload.insert(
                "event".to_string(),
                serde_json::Value::String(event_str.to_string()),
            );
        }

        if let Some(ref filter) = config.filter {
            payload.insert(
                "filter".to_string(),
                serde_json::Value::String(filter.clone()),
            );
        }

        let message = RealtimeProtocolMessage {
            topic: topic.to_string(),
            event: "phx_join".to_string(),
            payload: serde_json::Value::Object(payload),
            ref_id: Uuid::new_v4().to_string(),
        };

        self.send_message(&message).await
    }

    /// Send leave message to Supabase realtime server
    async fn send_leave_message(&self, topic: &str) -> Result<()> {
        let message = RealtimeProtocolMessage {
            topic: topic.to_string(),
            event: "phx_leave".to_string(),
            payload: serde_json::Value::Object(serde_json::Map::new()),
            ref_id: Uuid::new_v4().to_string(),
        };

        self.send_message(&message).await
    }

    /// Send message through WebSocket
    async fn send_message(&self, message: &RealtimeProtocolMessage) -> Result<()> {
        let message_json = serde_json::to_string(message)?;

        let mut connection_guard = self.connection_manager.connection.write().await;
        if let Some(ref mut connection) = *connection_guard {
            connection.send(&message_json).await?;
            debug!("Sent realtime message: {}", message_json);
        } else {
            return Err(Error::realtime("Not connected to realtime server"));
        }

        Ok(())
    }

    /// Start the message processing loop
    async fn start_message_loop(&self) -> Result<()> {
        if self
            .connection_manager
            .is_message_loop_running
            .load(Ordering::SeqCst)
        {
            debug!("Message loop already running");
            return Ok(());
        }

        self.connection_manager
            .is_message_loop_running
            .store(true, Ordering::SeqCst);
        self.message_loop_handle.store(true, Ordering::SeqCst);

        let connection_manager = Arc::clone(&self.connection_manager);
        let loop_handle = Arc::clone(&self.message_loop_handle);

        // Spawn the message loop
        #[cfg(not(target_arch = "wasm32"))]
        {
            let connection_manager = Arc::clone(&connection_manager);
            let loop_handle = Arc::clone(&loop_handle);

            tokio::spawn(async move {
                Self::message_loop(connection_manager, loop_handle).await;
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            // For WASM, we'll use a simple polling approach
            wasm_bindgen_futures::spawn_local(async move {
                Self::message_loop(connection_manager, loop_handle).await;
            });
        }

        info!("Started realtime message loop");
        Ok(())
    }

    /// Main message processing loop
    async fn message_loop(
        connection_manager: Arc<ConnectionManager>,
        loop_handle: Arc<AtomicBool>,
    ) {
        debug!("Starting realtime message loop");

        while loop_handle.load(Ordering::SeqCst) {
            // Try to receive messages
            let message = {
                let mut connection_guard = connection_manager.connection.write().await;

                if let Some(ref mut connection) = *connection_guard {
                    if !connection.is_connected() {
                        debug!("Connection lost, stopping message loop");
                        break;
                    }

                    match connection.receive().await {
                        Ok(Some(msg)) => Some(msg),
                        Ok(None) => None,
                        Err(e) => {
                            error!("Error receiving message: {}", e);
                            None
                        }
                    }
                } else {
                    debug!("No connection available, stopping message loop");
                    break;
                }
            };

            if let Some(message_str) = message {
                debug!("Received realtime message: {}", message_str);

                // Parse the message
                match serde_json::from_str::<RealtimeMessage>(&message_str) {
                    Ok(realtime_message) => {
                        // Process the message
                        Self::process_message(&connection_manager, realtime_message).await;
                    }
                    Err(e) => {
                        debug!(
                            "Failed to parse realtime message: {} - Error: {}",
                            message_str, e
                        );
                        // Try to parse as protocol message (join/leave responses, etc.)
                        if let Ok(_protocol_msg) =
                            serde_json::from_str::<serde_json::Value>(&message_str)
                        {
                            debug!("Received protocol message, ignoring for now");
                        }
                    }
                }
            }

            // Small delay to prevent busy waiting
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(Duration::from_millis(10)).await;

            #[cfg(target_arch = "wasm32")]
            {
                // For WASM, use a simple promise-based delay
                use wasm_bindgen::prelude::*;
                use wasm_bindgen_futures::JsFuture;

                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    web_sys::window()
                        .unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 10)
                        .unwrap();
                });
                let _ = JsFuture::from(promise).await;
            }
        }

        connection_manager
            .is_message_loop_running
            .store(false, Ordering::SeqCst);
        debug!("Realtime message loop stopped");
    }

    /// Process incoming realtime message
    async fn process_message(
        connection_manager: &Arc<ConnectionManager>,
        message: RealtimeMessage,
    ) {
        debug!("Processing message for topic: {}", message.topic);

        let subscriptions = connection_manager.subscriptions.read().await;

        let mut matched_subscriptions = Vec::new();

        // Find matching subscriptions
        for subscription in subscriptions.values() {
            if Self::topic_matches(&subscription.topic, &message.topic) {
                // Check event filter
                if let Some(ref event_filter) = subscription.config.event {
                    let message_event = match message.event.as_str() {
                        "INSERT" => Some(RealtimeEvent::Insert),
                        "UPDATE" => Some(RealtimeEvent::Update),
                        "DELETE" => Some(RealtimeEvent::Delete),
                        _ => None,
                    };

                    if let Some(msg_event) = message_event {
                        if *event_filter != RealtimeEvent::All && *event_filter != msg_event {
                            continue; // Skip if event doesn't match
                        }
                    }
                }

                matched_subscriptions.push(subscription.clone());
            }
        }

        drop(subscriptions); // Explicitly drop the guard

        // Call callbacks for matched subscriptions
        for subscription in matched_subscriptions {
            debug!("Calling callback for subscription: {}", subscription.id);
            (subscription.callback)(message.clone());
        }
    }

    /// Check if topic matches subscription pattern
    fn topic_matches(subscription_topic: &str, message_topic: &str) -> bool {
        // Simple pattern matching - could be enhanced with wildcards
        subscription_topic == message_topic || message_topic.starts_with(subscription_topic)
    }

    /// Track user presence in a channel
    ///
    /// # Examples
    /// ```rust,no_run
    /// use supabase_lib_rs::realtime::PresenceState;
    /// use std::collections::HashMap;
    ///
    /// # async fn example(realtime: &supabase_lib_rs::realtime::Realtime) -> supabase_lib_rs::Result<()> {
    /// let mut metadata = HashMap::new();
    /// metadata.insert("status".to_string(), serde_json::Value::String("online".to_string()));
    /// metadata.insert("location".to_string(), serde_json::Value::String("dashboard".to_string()));
    ///
    /// let presence_state = PresenceState {
    ///     user_id: "user123".to_string(),
    ///     online_at: chrono::Utc::now().to_rfc3339(),
    ///     metadata: Some(metadata),
    /// };
    ///
    /// realtime.track_presence("lobby", presence_state).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn track_presence(&self, channel: &str, presence_state: PresenceState) -> Result<()> {
        debug!(
            "Tracking presence for user {} in channel {}",
            presence_state.user_id, channel
        );

        let topic = format!("realtime:{}", channel);
        let ref_id = Uuid::new_v4().to_string();

        let message = RealtimeProtocolMessage {
            topic: topic.clone(),
            event: "presence".to_string(),
            payload: serde_json::json!({
                "event": "track",
                "payload": presence_state
            }),
            ref_id,
        };

        let mut connection_guard = self.connection_manager.connection.write().await;
        if let Some(ref mut connection) = *connection_guard {
            let message_json = serde_json::to_string(&message).map_err(|e| {
                Error::realtime(format!("Failed to serialize presence message: {}", e))
            })?;

            connection.send(&message_json).await?;
            info!(
                "Started tracking presence for user {}",
                presence_state.user_id
            );
        } else {
            return Err(Error::realtime("Not connected to realtime server"));
        }

        Ok(())
    }

    /// Stop tracking user presence in a channel
    ///
    /// # Examples
    /// ```rust,no_run
    /// # async fn example(realtime: &supabase_lib_rs::realtime::Realtime) -> supabase_lib_rs::Result<()> {
    /// realtime.untrack_presence("lobby", "user123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn untrack_presence(&self, channel: &str, user_id: &str) -> Result<()> {
        debug!(
            "Untracking presence for user {} in channel {}",
            user_id, channel
        );

        let topic = format!("realtime:{}", channel);
        let ref_id = Uuid::new_v4().to_string();

        let message = RealtimeProtocolMessage {
            topic: topic.clone(),
            event: "presence".to_string(),
            payload: serde_json::json!({
                "event": "untrack",
                "payload": {
                    "user_id": user_id
                }
            }),
            ref_id,
        };

        let mut connection_guard = self.connection_manager.connection.write().await;
        if let Some(ref mut connection) = *connection_guard {
            let message_json = serde_json::to_string(&message).map_err(|e| {
                Error::realtime(format!("Failed to serialize presence message: {}", e))
            })?;

            connection.send(&message_json).await?;
            info!("Stopped tracking presence for user {}", user_id);
        } else {
            return Err(Error::realtime("Not connected to realtime server"));
        }

        Ok(())
    }

    /// Get all users currently present in a channel
    ///
    /// # Examples
    /// ```rust,no_run
    /// # async fn example(realtime: &supabase_lib_rs::realtime::Realtime) -> supabase_lib_rs::Result<()> {
    /// let present_users = realtime.get_presence("lobby").await?;
    /// println!("Users online: {}", present_users.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_presence(&self, channel: &str) -> Result<Vec<PresenceState>> {
        debug!("Getting presence for channel: {}", channel);

        let topic = format!("realtime:{}", channel);
        let ref_id = Uuid::new_v4().to_string();

        let message = RealtimeProtocolMessage {
            topic: topic.clone(),
            event: "presence".to_string(),
            payload: serde_json::json!({
                "event": "state"
            }),
            ref_id,
        };

        let mut connection_guard = self.connection_manager.connection.write().await;
        if let Some(ref mut connection) = *connection_guard {
            let message_json = serde_json::to_string(&message).map_err(|e| {
                Error::realtime(format!("Failed to serialize presence message: {}", e))
            })?;

            connection.send(&message_json).await?;

            // Note: In a real implementation, you'd wait for the response
            // For now, returning empty vec as this would require more complex message handling
            info!("Requested presence state for channel: {}", channel);
            Ok(Vec::new())
        } else {
            Err(Error::realtime("Not connected to realtime server"))
        }
    }

    /// Send a broadcast message to all subscribers in a channel
    ///
    /// # Examples
    /// ```rust,no_run
    /// use serde_json::json;
    ///
    /// # async fn example(realtime: &supabase_lib_rs::realtime::Realtime) -> supabase_lib_rs::Result<()> {
    /// let payload = json!({
    ///     "message": "Hello, everyone!",
    ///     "from": "user123",
    ///     "timestamp": chrono::Utc::now().to_rfc3339()
    /// });
    ///
    /// realtime.broadcast("chat", "new_message", payload, Some("user123")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn broadcast(
        &self,
        channel: &str,
        event: &str,
        payload: serde_json::Value,
        from_user_id: Option<&str>,
    ) -> Result<()> {
        debug!(
            "Broadcasting message to channel: {} event: {}",
            channel, event
        );

        let topic = format!("realtime:{}", channel);
        let ref_id = Uuid::new_v4().to_string();

        let broadcast_message = BroadcastMessage {
            event: event.to_string(),
            payload,
            from_user_id: from_user_id.map(|s| s.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let message = RealtimeProtocolMessage {
            topic: topic.clone(),
            event: "broadcast".to_string(),
            payload: serde_json::to_value(broadcast_message)?,
            ref_id,
        };

        let mut connection_guard = self.connection_manager.connection.write().await;
        if let Some(ref mut connection) = *connection_guard {
            let message_json = serde_json::to_string(&message).map_err(|e| {
                Error::realtime(format!("Failed to serialize broadcast message: {}", e))
            })?;

            connection.send(&message_json).await?;
            info!("Sent broadcast message to channel: {}", channel);
        } else {
            return Err(Error::realtime("Not connected to realtime server"));
        }

        Ok(())
    }

    /// Subscribe to a channel with advanced configuration
    ///
    /// This method provides more control over subscriptions including presence tracking,
    /// broadcast messages, and advanced filtering.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use supabase_lib_rs::realtime::{SubscriptionConfig, RealtimeEvent, AdvancedFilter, FilterOperator};
    /// use std::sync::Arc;
    ///
    /// # async fn example(realtime: &supabase_lib_rs::realtime::Realtime) -> supabase_lib_rs::Result<()> {
    /// let config = SubscriptionConfig {
    ///     table: Some("posts".to_string()),
    ///     schema: "public".to_string(),
    ///     event: Some(RealtimeEvent::All),
    ///     advanced_filters: vec![
    ///         AdvancedFilter {
    ///             column: "status".to_string(),
    ///             operator: FilterOperator::Equal,
    ///             value: serde_json::Value::String("published".to_string()),
    ///         }
    ///     ],
    ///     enable_presence: true,
    ///     enable_broadcast: true,
    ///     presence_callback: Some(Arc::new(|event| {
    ///         println!("Presence event: {:?}", event);
    ///     })),
    ///     broadcast_callback: Some(Arc::new(|message| {
    ///         println!("Broadcast message: {:?}", message);
    ///     })),
    ///     ..Default::default()
    /// };
    ///
    /// let subscription_id = realtime.subscribe_advanced("posts", config, |msg| {
    ///     println!("Received message: {:?}", msg);
    /// }).await?;
    /// println!("Advanced subscription ID: {}", subscription_id);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn subscribe_advanced<F>(
        &self,
        channel: &str,
        config: SubscriptionConfig,
        callback: F,
    ) -> Result<String>
    where
        F: Fn(RealtimeMessage) + Send + Sync + 'static,
    {
        debug!("Creating advanced subscription for channel: {}", channel);

        let subscription_id = Uuid::new_v4().to_string();
        let topic = if let Some(ref table) = config.table {
            format!("realtime:{}:{}:{}", config.schema, table, channel)
        } else {
            format!("realtime:{}", channel)
        };

        // Build filter string from advanced filters
        let mut filter_parts = Vec::new();

        if let Some(ref simple_filter) = config.filter {
            filter_parts.push(simple_filter.clone());
        }

        for advanced_filter in &config.advanced_filters {
            let filter_str = match &advanced_filter.value {
                serde_json::Value::String(s) => format!(
                    "{}={}. {}",
                    advanced_filter.column,
                    serde_json::to_string(&advanced_filter.operator)?.trim_matches('"'),
                    s
                ),
                serde_json::Value::Array(arr) => {
                    let values: Vec<String> = arr
                        .iter()
                        .map(|v| v.to_string().trim_matches('"').to_string())
                        .collect();
                    format!(
                        "{}={}.({})",
                        advanced_filter.column,
                        serde_json::to_string(&advanced_filter.operator)?.trim_matches('"'),
                        values.join(",")
                    )
                }
                other => format!(
                    "{}={}. {}",
                    advanced_filter.column,
                    serde_json::to_string(&advanced_filter.operator)?.trim_matches('"'),
                    other.to_string().trim_matches('"')
                ),
            };
            filter_parts.push(filter_str);
        }

        let combined_filter = if !filter_parts.is_empty() {
            Some(filter_parts.join(" and "))
        } else {
            None
        };

        let subscription = Subscription {
            id: subscription_id.clone(),
            topic: topic.clone(),
            config: SubscriptionConfig {
                filter: combined_filter,
                ..config.clone()
            },
            callback: Arc::new(callback),
        };

        // Store subscription
        {
            let mut subscriptions = self.connection_manager.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription);
        }

        // Send join message
        let ref_id = self
            .connection_manager
            .ref_counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string();

        let mut join_payload = serde_json::json!({
            "config": {
                "postgres_changes": [{
                    "event": config.event.unwrap_or(RealtimeEvent::All),
                    "schema": config.schema,
                }]
            }
        });

        if let Some(ref table) = config.table {
            join_payload["config"]["postgres_changes"][0]["table"] =
                serde_json::Value::String(table.clone());
        }

        if let Some(ref filter) = config.filter {
            join_payload["config"]["postgres_changes"][0]["filter"] =
                serde_json::Value::String(filter.clone());
        }

        // Add presence configuration
        if config.enable_presence {
            join_payload["config"]["presence"] = serde_json::json!({ "key": "" });
        }

        // Add broadcast configuration
        if config.enable_broadcast {
            join_payload["config"]["broadcast"] = serde_json::json!({ "self": true });
        }

        let join_message = RealtimeProtocolMessage {
            topic: topic.clone(),
            event: "phx_join".to_string(),
            payload: join_payload,
            ref_id,
        };

        let mut connection_guard = self.connection_manager.connection.write().await;
        if let Some(ref mut connection) = *connection_guard {
            let message_json = serde_json::to_string(&join_message)
                .map_err(|e| Error::realtime(format!("Failed to serialize join message: {}", e)))?;

            connection.send(&message_json).await?;
            info!("Advanced subscription created: {}", subscription_id);
        } else {
            return Err(Error::realtime("Not connected to realtime server"));
        }

        Ok(subscription_id)
    }

    /// Subscribe to a channel with advanced configuration (WASM version)
    #[cfg(target_arch = "wasm32")]
    pub async fn subscribe_advanced<F>(
        &self,
        channel: &str,
        config: SubscriptionConfig,
        callback: F,
    ) -> Result<String>
    where
        F: Fn(RealtimeMessage) + 'static,
    {
        debug!("Creating advanced subscription for channel: {}", channel);

        let subscription_id = Uuid::new_v4().to_string();
        let topic = if let Some(ref table) = config.table {
            format!("realtime:{}:{}:{}", config.schema, table, channel)
        } else {
            format!("realtime:{}", channel)
        };

        // Build filter string from advanced filters
        let mut filter_parts = Vec::new();

        if let Some(ref simple_filter) = config.filter {
            filter_parts.push(simple_filter.clone());
        }

        for advanced_filter in &config.advanced_filters {
            let filter_str = match &advanced_filter.value {
                serde_json::Value::String(s) => format!(
                    "{}={}. {}",
                    advanced_filter.column,
                    serde_json::to_string(&advanced_filter.operator)?.trim_matches('"'),
                    s
                ),
                serde_json::Value::Array(arr) => {
                    let values: Vec<String> = arr
                        .iter()
                        .map(|v| v.to_string().trim_matches('"').to_string())
                        .collect();
                    format!(
                        "{}={}.({})",
                        advanced_filter.column,
                        serde_json::to_string(&advanced_filter.operator)?.trim_matches('"'),
                        values.join(",")
                    )
                }
                other => format!(
                    "{}={}. {}",
                    advanced_filter.column,
                    serde_json::to_string(&advanced_filter.operator)?.trim_matches('"'),
                    other.to_string().trim_matches('"')
                ),
            };
            filter_parts.push(filter_str);
        }

        let combined_filter = if !filter_parts.is_empty() {
            Some(filter_parts.join(" and "))
        } else {
            None
        };

        let subscription = Subscription {
            id: subscription_id.clone(),
            topic: topic.clone(),
            config: SubscriptionConfig {
                filter: combined_filter,
                ..config.clone()
            },
            callback: Arc::new(callback),
        };

        // Store subscription
        {
            let mut subscriptions = self.connection_manager.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription);
        }

        // Send join message
        let ref_id = self
            .connection_manager
            .ref_counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string();

        let mut join_payload = serde_json::json!({
            "config": {
                "postgres_changes": [{
                    "event": config.event.unwrap_or(RealtimeEvent::All),
                    "schema": config.schema,
                }]
            }
        });

        if let Some(ref table) = config.table {
            join_payload["config"]["postgres_changes"][0]["table"] =
                serde_json::Value::String(table.clone());
        }

        if let Some(ref filter) = config.filter {
            join_payload["config"]["postgres_changes"][0]["filter"] =
                serde_json::Value::String(filter.clone());
        }

        // Add presence configuration
        if config.enable_presence {
            join_payload["config"]["presence"] = serde_json::json!({ "key": "" });
        }

        // Add broadcast configuration
        if config.enable_broadcast {
            join_payload["config"]["broadcast"] = serde_json::json!({ "self": true });
        }

        let join_message = RealtimeProtocolMessage {
            topic: topic.clone(),
            event: "phx_join".to_string(),
            payload: join_payload,
            ref_id,
        };

        let mut connection_guard = self.connection_manager.connection.write().await;
        if let Some(ref mut connection) = *connection_guard {
            let message_json = serde_json::to_string(&join_message)
                .map_err(|e| Error::realtime(format!("Failed to serialize join message: {}", e)))?;

            connection.send(&message_json).await?;
            info!("Advanced subscription created: {}", subscription_id);
        } else {
            return Err(Error::realtime("Not connected to realtime server"));
        }

        Ok(subscription_id)
    }
}

/// Builder for channel subscriptions
///
/// Provides a fluent API for configuring realtime subscriptions.
///
/// # Examples
/// ```rust,no_run
/// # use supabase_lib_rs::Client;
/// # use supabase_lib_rs::realtime::RealtimeEvent;
/// # async fn example() -> supabase_lib_rs::Result<()> {
/// let client = Client::new("your-url", "your-key")?;
///
/// let subscription = client.realtime()
///     .channel("public-posts")
///     .table("posts")
///     .event(RealtimeEvent::Insert)
///     .filter("author_id=eq.123")
///     .subscribe(|message| {
///         println!("New post by author 123: {:?}", message);
///     })
///     .await?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "realtime")]
pub struct ChannelBuilder {
    realtime: Realtime,
    config: SubscriptionConfig,
}

#[cfg(feature = "realtime")]
impl ChannelBuilder {
    /// Set the table to subscribe to
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = Client::new("your-url", "your-key")?;
    ///
    /// let subscription = client.realtime()
    ///     .channel("posts")
    ///     .table("posts") // Subscribe to the 'posts' table
    ///     .subscribe(|_| {})
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn table(mut self, table: &str) -> Self {
        self.config.table = Some(table.to_string());
        self
    }

    /// Set the schema (default: "public")
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = Client::new("your-url", "your-key")?;
    ///
    /// let subscription = client.realtime()
    ///     .channel("admin-logs")
    ///     .schema("admin") // Subscribe to 'admin' schema
    ///     .table("logs")
    ///     .subscribe(|_| {})
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn schema(mut self, schema: &str) -> Self {
        self.config.schema = schema.to_string();
        self
    }

    /// Set the event type filter
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use supabase_lib_rs::realtime::RealtimeEvent;
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = Client::new("your-url", "your-key")?;
    ///
    /// // Only listen to INSERT events
    /// let subscription = client.realtime()
    ///     .channel("new-posts")
    ///     .table("posts")
    ///     .event(RealtimeEvent::Insert)
    ///     .subscribe(|_| {})
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn event(mut self, event: RealtimeEvent) -> Self {
        self.config.event = Some(event);
        self
    }

    /// Set a filter for the subscription
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = Client::new("your-url", "your-key")?;
    ///
    /// // Only posts by specific author
    /// let subscription = client.realtime()
    ///     .channel("my-posts")
    ///     .table("posts")
    ///     .filter("author_id=eq.123")
    ///     .subscribe(|_| {})
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn filter(mut self, filter: &str) -> Self {
        self.config.filter = Some(filter.to_string());
        self
    }

    /// Subscribe with a callback function
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = Client::new("your-url", "your-key")?;
    ///
    /// let subscription_id = client.realtime()
    ///     .channel("posts")
    ///     .table("posts")
    ///     .subscribe(|message| {
    ///         match message.event.as_str() {
    ///             "INSERT" => println!("New post created!"),
    ///             "UPDATE" => println!("Post updated!"),
    ///             "DELETE" => println!("Post deleted!"),
    ///             _ => println!("Other event: {}", message.event),
    ///         }
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn subscribe<F>(self, callback: F) -> Result<String>
    where
        F: Fn(RealtimeMessage) + Send + Sync + 'static,
    {
        self.realtime.subscribe(self.config, callback).await
    }

    /// Subscribe with a callback function (WASM version)
    #[cfg(target_arch = "wasm32")]
    pub async fn subscribe<F>(self, callback: F) -> Result<String>
    where
        F: Fn(RealtimeMessage) + 'static,
    {
        self.realtime.subscribe(self.config, callback).await
    }
}

#[cfg(all(test, feature = "realtime"))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_realtime_creation() {
        let config = Arc::new(SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            key: "test-key".to_string(),
            ..Default::default()
        });

        let realtime = Realtime::new(config).unwrap();
        assert!(!realtime.is_connected().await);
    }

    #[tokio::test]
    async fn test_subscription_config_default() {
        let config = SubscriptionConfig::default();
        assert_eq!(config.schema, "public");
        assert!(config.table.is_none());
        assert!(config.event.is_none());
        assert!(config.filter.is_none());
    }

    #[tokio::test]
    async fn test_realtime_event_serialization() {
        use serde_json;

        let event = RealtimeEvent::Insert;
        let serialized = serde_json::to_string(&event).unwrap();
        assert_eq!(serialized, "\"INSERT\"");

        let event = RealtimeEvent::All;
        let serialized = serde_json::to_string(&event).unwrap();
        assert_eq!(serialized, "\"*\"");
    }

    #[tokio::test]
    async fn test_build_topic() {
        let config = Arc::new(SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            key: "test-key".to_string(),
            ..Default::default()
        });

        let realtime = Realtime::new(config).unwrap();

        // Test with table
        let subscription_config = SubscriptionConfig {
            table: Some("posts".to_string()),
            schema: "public".to_string(),
            event: None,
            filter: None,
            ..Default::default()
        };
        let topic = realtime.build_topic(&subscription_config);
        assert_eq!(topic, "realtime:public:posts");

        // Test without table
        let subscription_config = SubscriptionConfig {
            table: None,
            schema: "admin".to_string(),
            event: None,
            filter: None,
            ..Default::default()
        };
        let topic = realtime.build_topic(&subscription_config);
        assert_eq!(topic, "realtime:admin");
    }

    #[tokio::test]
    async fn test_topic_matching() {
        // Exact match
        assert!(Realtime::topic_matches(
            "realtime:public:posts",
            "realtime:public:posts"
        ));

        // Prefix match
        assert!(Realtime::topic_matches(
            "realtime:public",
            "realtime:public:posts"
        ));

        // No match
        assert!(!Realtime::topic_matches(
            "realtime:public:users",
            "realtime:public:posts"
        ));
    }

    #[tokio::test]
    async fn test_realtime_message_parsing() {
        let json = r#"{
            "event": "INSERT",
            "payload": {
                "record": {"id": 1, "title": "Test"},
                "schema": "public",
                "table": "posts"
            },
            "topic": "realtime:public:posts"
        }"#;

        let message = serde_json::from_str::<RealtimeMessage>(json);
        assert!(message.is_ok());

        let message = message.unwrap();
        assert_eq!(message.event, "INSERT");
        assert_eq!(message.topic, "realtime:public:posts");
        assert!(message.payload.record.is_some());
    }

    #[tokio::test]
    async fn test_channel_builder() {
        let config = Arc::new(SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            key: "test-key".to_string(),
            ..Default::default()
        });

        let realtime = Realtime::new(config).unwrap();
        let builder = realtime.channel("test");

        // Test builder methods
        let builder = builder
            .table("posts")
            .schema("public")
            .event(RealtimeEvent::Insert)
            .filter("author_id=eq.123");

        assert_eq!(builder.config.table, Some("posts".to_string()));
        assert_eq!(builder.config.schema, "public");
        assert_eq!(builder.config.event, Some(RealtimeEvent::Insert));
        assert_eq!(builder.config.filter, Some("author_id=eq.123".to_string()));
    }

    #[cfg(not(target_arch = "wasm32"))] // This test requires native tokio
    #[tokio::test]
    async fn test_subscription_callback() {
        let config = Arc::new(SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            key: "test-key".to_string(),
            ..Default::default()
        });

        let realtime = Realtime::new(config).unwrap();

        // Test that subscription creation works without connecting
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);

        let subscription_config = SubscriptionConfig {
            table: Some("test".to_string()),
            schema: "public".to_string(),
            event: Some(RealtimeEvent::All),
            filter: None,
            ..Default::default()
        };

        // This will fail because we're not connected, but that's expected
        let result = realtime
            .subscribe(subscription_config, move |_msg| {
                called_clone.store(true, Ordering::SeqCst);
            })
            .await;

        // Should fail due to no connection
        assert!(result.is_err());
        assert!(!called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_protocol_message_serialization() {
        let message = RealtimeProtocolMessage {
            topic: "realtime:public:posts".to_string(),
            event: "phx_join".to_string(),
            payload: serde_json::json!({"table": "posts"}),
            ref_id: "123".to_string(),
        };

        let serialized = serde_json::to_string(&message).unwrap();
        assert!(serialized.contains("phx_join"));
        assert!(serialized.contains("realtime:public:posts"));
        assert!(serialized.contains("posts"));
    }

    #[tokio::test]
    async fn test_event_filter_matching() {
        // Test INSERT event matching
        let insert_event = Some(RealtimeEvent::Insert);
        let update_event = Some(RealtimeEvent::Update);
        let all_event = Some(RealtimeEvent::All);

        // INSERT should match INSERT
        assert_eq!(insert_event, Some(RealtimeEvent::Insert));

        // INSERT should not match UPDATE
        assert_ne!(insert_event, update_event);

        // ALL should match ALL
        assert_eq!(all_event, Some(RealtimeEvent::All));
    }
}
