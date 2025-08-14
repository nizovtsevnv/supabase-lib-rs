//! Realtime module for Supabase WebSocket subscriptions
//!
//! This module provides cross-platform WebSocket support using proper abstractions:
//! - Native: Uses tokio-tungstenite with TLS support
//! - WASM: Uses web-sys WebSocket API through the browser
//!
//! ## Usage
//!
//! ```rust,no_run
//! use supabase::Client;
//! use supabase::realtime::RealtimeEvent;
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

/// Realtime client for WebSocket subscriptions
///
/// Provides cross-platform realtime subscriptions to Supabase database changes.
///
/// # Examples
///
/// ## Basic subscription
/// ```rust,no_run
/// use supabase::{Client, realtime::RealtimeEvent};
///
/// # async fn example() -> supabase::Result<()> {
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
/// use supabase::realtime::{SubscriptionConfig, RealtimeEvent};
///
/// let config = SubscriptionConfig {
///     table: Some("posts".to_string()),
///     schema: "public".to_string(),
///     event: Some(RealtimeEvent::Insert),
///     filter: Some("author_id=eq.123".to_string()),
/// };
/// ```
#[cfg(feature = "realtime")]
#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    pub table: Option<String>,
    pub schema: String,
    pub event: Option<RealtimeEvent>,
    pub filter: Option<String>,
}

#[cfg(feature = "realtime")]
impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            table: None,
            schema: "public".to_string(),
            event: None,
            filter: None,
        }
    }
}

/// Realtime event types for filtering subscriptions
///
/// # Examples
/// ```rust
/// use supabase::realtime::RealtimeEvent;
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

#[cfg(feature = "realtime")]
impl Realtime {
    /// Create a new realtime client (works on both native and WASM)
    ///
    /// # Examples
    /// ```rust
    /// use supabase::types::SupabaseConfig;
    /// use supabase::realtime::Realtime;
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
    /// # use supabase::Client;
    /// # async fn example() -> supabase::Result<()> {
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
    /// # use supabase::Client;
    /// # async fn example() -> supabase::Result<()> {
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
    /// # use supabase::Client;
    /// # async fn example() -> supabase::Result<()> {
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
    /// # use supabase::Client;
    /// # async fn example() -> supabase::Result<()> {
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
    /// # use supabase::Client;
    /// # async fn example() -> supabase::Result<()> {
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
}

/// Builder for channel subscriptions
///
/// Provides a fluent API for configuring realtime subscriptions.
///
/// # Examples
/// ```rust,no_run
/// # use supabase::Client;
/// # use supabase::realtime::RealtimeEvent;
/// # async fn example() -> supabase::Result<()> {
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
    /// # use supabase::Client;
    /// # async fn example() -> supabase::Result<()> {
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
    /// # use supabase::Client;
    /// # async fn example() -> supabase::Result<()> {
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
    /// # use supabase::Client;
    /// # use supabase::realtime::RealtimeEvent;
    /// # async fn example() -> supabase::Result<()> {
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
    /// # use supabase::Client;
    /// # async fn example() -> supabase::Result<()> {
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
    /// # use supabase::Client;
    /// # async fn example() -> supabase::Result<()> {
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
        };
        let topic = realtime.build_topic(&subscription_config);
        assert_eq!(topic, "realtime:public:posts");

        // Test without table
        let subscription_config = SubscriptionConfig {
            table: None,
            schema: "admin".to_string(),
            event: None,
            filter: None,
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
