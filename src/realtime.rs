//! Realtime module for Supabase WebSocket subscriptions

use crate::{
    error::{Error, Result},
    types::SupabaseConfig,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    net::TcpStream,
    sync::{mpsc, RwLock},
    time::{interval, sleep},
};
use tokio_tungstenite::{
    connect_async, tungstenite::Message as WsMessage, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, warn};
use url::Url;
use uuid::Uuid;

/// Realtime client for WebSocket subscriptions
#[derive(Debug, Clone)]
pub struct Realtime {
    #[allow(dead_code)]
    config: Arc<SupabaseConfig>,
    connection_manager: Arc<ConnectionManager>,
}

/// Connection manager for WebSocket connections
#[derive(Debug)]
struct ConnectionManager {
    url: String,
    api_key: String,
    connection: RwLock<Option<Connection>>,
    ref_counter: AtomicU64,
    subscriptions: RwLock<HashMap<String, Subscription>>,
    #[allow(dead_code)]
    message_sender: Option<mpsc::UnboundedSender<RealtimeMessage>>,
}

/// WebSocket connection wrapper
#[derive(Debug)]
struct Connection {
    sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>,
    receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    is_connected: bool,
}

/// Subscription information
#[derive(Clone)]
pub struct Subscription {
    pub id: String,
    pub topic: String,
    pub config: SubscriptionConfig,
    pub callback: Arc<dyn Fn(RealtimeMessage) + Send + Sync>,
}

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
#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    pub table: Option<String>,
    pub schema: String,
    pub event: Option<RealtimeEvent>,
    pub filter: Option<String>,
}

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

/// Realtime event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RealtimeEvent {
    Insert,
    Update,
    Delete,
    Select,
    All,
}

/// Realtime message from Supabase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMessage {
    pub topic: String,
    pub event: String,
    pub payload: RealtimePayload,
    #[serde(rename = "ref")]
    pub ref_id: Option<String>,
}

/// Payload structure for realtime messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimePayload {
    pub schema: Option<String>,
    pub table: Option<String>,
    pub commit_timestamp: Option<String>,
    pub event_type: Option<String>,
    pub new: Option<serde_json::Value>,
    pub old: Option<serde_json::Value>,
    pub columns: Option<Vec<ColumnInfo>>,
    pub errors: Option<Vec<String>>,
}

/// Column information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub type_name: String,
    pub flags: Option<Vec<String>>,
}

/// Phoenix channel message structure
#[derive(Debug, Serialize, Deserialize)]
struct PhoenixMessage {
    topic: String,
    event: String,
    payload: serde_json::Value,
    #[serde(rename = "ref")]
    ref_id: String,
}

/// Subscription builder
pub struct SubscriptionBuilder {
    realtime: Realtime,
    config: SubscriptionConfig,
}

impl Realtime {
    /// Create a new Realtime instance
    pub fn new(config: Arc<SupabaseConfig>) -> Result<Self> {
        debug!("Initializing Realtime module");

        let ws_url = config
            .url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let realtime_url = format!("{}/realtime/v1/websocket", ws_url);

        let connection_manager = Arc::new(ConnectionManager {
            url: realtime_url,
            api_key: config.key.clone(),
            connection: RwLock::new(None),
            ref_counter: AtomicU64::new(1),
            subscriptions: RwLock::new(HashMap::new()),
            message_sender: None,
        });

        Ok(Self {
            config,
            connection_manager,
        })
    }

    /// Connect to the realtime WebSocket
    pub async fn connect(&self) -> Result<()> {
        debug!("Connecting to realtime WebSocket");

        let mut url = Url::parse(&self.connection_manager.url)?;
        url.query_pairs_mut()
            .append_pair("apikey", &self.connection_manager.api_key)
            .append_pair("vsn", "1.0.0");

        let (ws_stream, response) = connect_async(url.as_str())
            .await
            .map_err(|e| Error::realtime(format!("WebSocket connection failed: {}", e)))?;

        info!("Connected to realtime WebSocket: {}", response.status());

        let (sender, receiver) = ws_stream.split();

        let connection = Connection {
            sender,
            receiver,
            is_connected: true,
        };

        let mut conn_guard = self.connection_manager.connection.write().await;
        *conn_guard = Some(connection);

        // Start message handling task
        self.start_message_handler().await?;

        // Send join message to establish connection
        self.send_join_message().await?;

        Ok(())
    }

    /// Disconnect from the realtime WebSocket
    pub async fn disconnect(&self) -> Result<()> {
        debug!("Disconnecting from realtime WebSocket");

        let mut conn_guard = self.connection_manager.connection.write().await;
        if let Some(mut connection) = conn_guard.take() {
            connection
                .sender
                .close()
                .await
                .map_err(|e| Error::realtime(format!("Failed to close WebSocket: {}", e)))?;
        }

        info!("Disconnected from realtime WebSocket");
        Ok(())
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let conn_guard = self.connection_manager.connection.read().await;
        conn_guard.as_ref().is_some_and(|c| c.is_connected)
    }

    /// Create a new subscription builder
    pub fn channel(&self, topic: &str) -> SubscriptionBuilder {
        SubscriptionBuilder {
            realtime: self.clone(),
            config: SubscriptionConfig {
                table: Some(topic.to_string()),
                ..Default::default()
            },
        }
    }

    /// Subscribe to table changes
    pub async fn subscribe<F>(&self, config: SubscriptionConfig, callback: F) -> Result<String>
    where
        F: Fn(RealtimeMessage) + Send + Sync + 'static,
    {
        let subscription_id = Uuid::new_v4().to_string();
        let topic = self.build_topic(&config);

        debug!(
            "Subscribing to topic: {} with ID: {}",
            topic, subscription_id
        );

        let subscription = Subscription {
            id: subscription_id.clone(),
            topic: topic.clone(),
            config,
            callback: Arc::new(callback),
        };

        // Add subscription to manager
        let mut subs_guard = self.connection_manager.subscriptions.write().await;
        subs_guard.insert(subscription_id.clone(), subscription);
        drop(subs_guard);

        // Send subscription message
        self.send_subscription_message(&topic, &subscription_id)
            .await?;

        info!("Subscribed successfully with ID: {}", subscription_id);
        Ok(subscription_id)
    }

    /// Unsubscribe from a topic
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        debug!("Unsubscribing from subscription: {}", subscription_id);

        let mut subs_guard = self.connection_manager.subscriptions.write().await;
        if let Some(subscription) = subs_guard.remove(subscription_id) {
            drop(subs_guard);
            self.send_unsubscription_message(&subscription.topic, subscription_id)
                .await?;
            info!("Unsubscribed successfully: {}", subscription_id);
        } else {
            warn!("Subscription not found: {}", subscription_id);
        }

        Ok(())
    }

    /// Build topic string from config
    fn build_topic(&self, config: &SubscriptionConfig) -> String {
        match &config.table {
            Some(table) => format!("realtime:{}:{}", config.schema, table),
            None => format!("realtime:{}", config.schema),
        }
    }

    /// Start the message handling task
    async fn start_message_handler(&self) -> Result<()> {
        let connection_manager = Arc::clone(&self.connection_manager);

        tokio::spawn(async move {
            let mut heartbeat_interval = interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    _ = heartbeat_interval.tick() => {
                        if let Err(e) = Self::send_heartbeat(&connection_manager).await {
                            error!("Failed to send heartbeat: {}", e);
                        }
                    }
                    result = Self::receive_message(&connection_manager) => {
                        match result {
                            Ok(Some(message)) => {
                                if let Err(e) = Self::handle_message(&connection_manager, message).await {
                                    error!("Failed to handle message: {}", e);
                                }
                            }
                            Ok(None) => {
                                debug!("Connection closed");
                                break;
                            }
                            Err(e) => {
                                error!("Error receiving message: {}", e);
                                sleep(Duration::from_secs(5)).await;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Send heartbeat message
    async fn send_heartbeat(manager: &ConnectionManager) -> Result<()> {
        let ref_id = manager
            .ref_counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string();

        let heartbeat = PhoenixMessage {
            topic: "phoenix".to_string(),
            event: "heartbeat".to_string(),
            payload: serde_json::Value::Object(serde_json::Map::new()),
            ref_id,
        };

        Self::send_phoenix_message(manager, heartbeat).await
    }

    /// Send join message
    async fn send_join_message(&self) -> Result<()> {
        let ref_id = self
            .connection_manager
            .ref_counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string();

        let join_message = PhoenixMessage {
            topic: "realtime:*".to_string(),
            event: "phx_join".to_string(),
            payload: serde_json::json!({}),
            ref_id,
        };

        Self::send_phoenix_message(&self.connection_manager, join_message).await
    }

    /// Send subscription message
    async fn send_subscription_message(&self, topic: &str, _subscription_id: &str) -> Result<()> {
        let ref_id = self
            .connection_manager
            .ref_counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string();

        let sub_message = PhoenixMessage {
            topic: topic.to_string(),
            event: "phx_join".to_string(),
            payload: serde_json::json!({
                "config": {
                    "postgres_changes": [{
                        "event": "*",
                        "schema": "public"
                    }]
                }
            }),
            ref_id,
        };

        Self::send_phoenix_message(&self.connection_manager, sub_message).await
    }

    /// Send unsubscription message
    async fn send_unsubscription_message(&self, topic: &str, _subscription_id: &str) -> Result<()> {
        let ref_id = self
            .connection_manager
            .ref_counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string();

        let unsub_message = PhoenixMessage {
            topic: topic.to_string(),
            event: "phx_leave".to_string(),
            payload: serde_json::Value::Object(serde_json::Map::new()),
            ref_id,
        };

        Self::send_phoenix_message(&self.connection_manager, unsub_message).await
    }

    /// Send phoenix message over WebSocket
    async fn send_phoenix_message(
        manager: &ConnectionManager,
        message: PhoenixMessage,
    ) -> Result<()> {
        let json_message = serde_json::to_string(&message)?;
        let ws_message = WsMessage::Text(json_message);

        let mut conn_guard = manager.connection.write().await;
        if let Some(ref mut connection) = conn_guard.as_mut() {
            connection
                .sender
                .send(ws_message)
                .await
                .map_err(|e| Error::realtime(format!("Failed to send message: {}", e)))?;
        } else {
            return Err(Error::realtime("No active connection"));
        }

        Ok(())
    }

    /// Receive message from WebSocket
    async fn receive_message(manager: &ConnectionManager) -> Result<Option<WsMessage>> {
        let mut conn_guard = manager.connection.write().await;
        if let Some(ref mut connection) = conn_guard.as_mut() {
            match connection.receiver.next().await {
                Some(Ok(message)) => Ok(Some(message)),
                Some(Err(e)) => Err(Error::realtime(format!("WebSocket error: {}", e))),
                None => Ok(None),
            }
        } else {
            Err(Error::realtime("No active connection"))
        }
    }

    /// Handle incoming message
    async fn handle_message(manager: &ConnectionManager, message: WsMessage) -> Result<()> {
        match message {
            WsMessage::Text(text) => {
                debug!("Received text message: {}", text);

                if let Ok(phoenix_msg) = serde_json::from_str::<PhoenixMessage>(&text) {
                    Self::handle_phoenix_message(manager, phoenix_msg).await?;
                }
            }
            WsMessage::Binary(_) => {
                debug!("Received binary message (ignoring)");
            }
            WsMessage::Close(_) => {
                info!("WebSocket connection closed");
            }
            WsMessage::Ping(_) | WsMessage::Pong(_) => {
                debug!("Received ping/pong");
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle Phoenix protocol message
    async fn handle_phoenix_message(
        manager: &ConnectionManager,
        message: PhoenixMessage,
    ) -> Result<()> {
        match message.event.as_str() {
            "postgres_changes" => {
                let realtime_message = RealtimeMessage {
                    topic: message.topic,
                    event: message.event,
                    payload: serde_json::from_value(message.payload).unwrap_or({
                        RealtimePayload {
                            schema: None,
                            table: None,
                            commit_timestamp: None,
                            event_type: None,
                            new: None,
                            old: None,
                            columns: None,
                            errors: None,
                        }
                    }),
                    ref_id: Some(message.ref_id),
                };

                // Find matching subscriptions and call callbacks
                let subs_guard = manager.subscriptions.read().await;
                for subscription in subs_guard.values() {
                    if subscription.topic == realtime_message.topic {
                        (subscription.callback)(realtime_message.clone());
                    }
                }
            }
            "phx_reply" => {
                debug!("Received reply: {:?}", message.payload);
            }
            "heartbeat" => {
                debug!("Heartbeat acknowledged");
            }
            _ => {
                debug!("Unhandled event: {}", message.event);
            }
        }

        Ok(())
    }
}

impl SubscriptionBuilder {
    /// Set the table to subscribe to
    pub fn table(mut self, table: &str) -> Self {
        self.config.table = Some(table.to_string());
        self
    }

    /// Set the schema
    pub fn schema(mut self, schema: &str) -> Self {
        self.config.schema = schema.to_string();
        self
    }

    /// Set the event type to listen for
    pub fn event(mut self, event: RealtimeEvent) -> Self {
        self.config.event = Some(event);
        self
    }

    /// Set a filter for the subscription
    pub fn filter(mut self, filter: &str) -> Self {
        self.config.filter = Some(filter.to_string());
        self
    }

    /// Subscribe with the configured options
    pub async fn subscribe<F>(self, callback: F) -> Result<String>
    where
        F: Fn(RealtimeMessage) + Send + Sync + 'static,
    {
        self.realtime.subscribe(self.config, callback).await
    }
}
