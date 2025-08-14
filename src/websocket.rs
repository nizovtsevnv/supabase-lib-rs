//! WebSocket abstraction for cross-platform compatibility
//!
//! This module provides WebSocket implementations for both native and WASM targets.
//! The abstractions allow the realtime module to work seamlessly across platforms.
//!
//! ## Platform Support
//!
//! - **Native**: Uses `tokio-tungstenite` with full TLS support
//! - **WASM**: Uses browser's `WebSocket` API through `web-sys`
//!
//! ## Usage
//!
//! This module is primarily used internally by the realtime module, but can be
//! used directly if needed:
//!
//! ```rust,ignore
//! use supabase::websocket::{create_websocket, WebSocketConnection};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut ws = create_websocket();
//! ws.connect("wss://example.com/websocket").await?;
//!
//! // Send a message
//! ws.send("Hello, WebSocket!").await?;
//!
//! // Receive messages
//! if let Some(message) = ws.receive().await? {
//!     println!("Received: {}", message);
//! }
//!
//! ws.close().await?;
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "realtime")]
use crate::error::{Error, Result};

/// Cross-platform WebSocket trait for native targets
///
/// This trait defines the interface for WebSocket connections on native platforms
/// where `Send + Sync` bounds are required for multi-threading.
#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
#[async_trait::async_trait]
pub trait WebSocketConnection: Send + Sync {
    /// Connect to a WebSocket server
    ///
    /// # Examples
    /// ```rust,ignore
    /// # use supabase::websocket::{create_websocket, WebSocketConnection};
    /// # async fn example() -> supabase::Result<()> {
    /// let mut ws = create_websocket();
    /// ws.connect("wss://echo.websocket.org").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn connect(&mut self, url: &str) -> Result<()>;

    /// Send a text message through the WebSocket
    ///
    /// # Examples
    /// ```rust,ignore
    /// # use supabase::websocket::{create_websocket, WebSocketConnection};
    /// # async fn example() -> supabase::Result<()> {
    /// let mut ws = create_websocket();
    /// ws.connect("wss://echo.websocket.org").await?;
    /// ws.send("Hello, server!").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn send(&mut self, message: &str) -> Result<()>;

    /// Receive a text message from the WebSocket
    ///
    /// Returns `Ok(Some(message))` if a message was received,
    /// `Ok(None)` if no message is available, or an error.
    ///
    /// # Examples
    /// ```rust,ignore
    /// # use supabase::websocket::{create_websocket, WebSocketConnection};
    /// # async fn example() -> supabase::Result<()> {
    /// let mut ws = create_websocket();
    /// ws.connect("wss://echo.websocket.org").await?;
    ///
    /// if let Some(message) = ws.receive().await? {
    ///     println!("Got message: {}", message);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn receive(&mut self) -> Result<Option<String>>;

    /// Close the WebSocket connection
    ///
    /// # Examples
    /// ```rust,ignore
    /// # use supabase::websocket::{create_websocket, WebSocketConnection};
    /// # async fn example() -> supabase::Result<()> {
    /// let mut ws = create_websocket();
    /// ws.connect("wss://echo.websocket.org").await?;
    /// // ... do work ...
    /// ws.close().await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn close(&mut self) -> Result<()>;

    /// Check if the WebSocket is currently connected
    ///
    /// # Examples
    /// ```rust,ignore
    /// # use supabase::websocket::{create_websocket, WebSocketConnection};
    /// # async fn example() -> supabase::Result<()> {
    /// let mut ws = create_websocket();
    /// assert!(!ws.is_connected()); // Not connected initially
    ///
    /// ws.connect("wss://echo.websocket.org").await?;
    /// assert!(ws.is_connected()); // Now connected
    /// # Ok(())
    /// # }
    /// ```
    fn is_connected(&self) -> bool;
}

/// Cross-platform WebSocket trait for WASM targets
///
/// This trait is the same as the native version but without `Send + Sync` bounds
/// since WASM runs in a single-threaded environment.
#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
#[async_trait::async_trait(?Send)]
pub trait WebSocketConnection {
    /// Connect to a WebSocket server
    async fn connect(&mut self, url: &str) -> Result<()>;
    /// Send a text message through the WebSocket
    async fn send(&mut self, message: &str) -> Result<()>;
    /// Receive a text message from the WebSocket
    async fn receive(&mut self) -> Result<Option<String>>;
    /// Close the WebSocket connection
    async fn close(&mut self) -> Result<()>;
    /// Check if the WebSocket is currently connected
    fn is_connected(&self) -> bool;
}

/// Native WebSocket implementation using tokio-tungstenite
///
/// This implementation provides full-featured WebSocket support for native platforms
/// including automatic TLS handling, connection management, and message processing.
///
/// ## Features
///
/// - Automatic TLS/SSL support for `wss://` URLs
/// - Connection state management
/// - Proper error handling and recovery
/// - Thread-safe operations
///
/// ## Example
///
/// ```rust,ignore
/// use supabase::websocket::NativeWebSocket;
/// use supabase::websocket::WebSocketConnection;
///
/// # async fn example() -> supabase::Result<()> {
/// let mut ws = NativeWebSocket::new();
/// ws.connect("wss://echo.websocket.org").await?;
///
/// ws.send("Hello from Rust!").await?;
/// if let Some(response) = ws.receive().await? {
///     println!("Server replied: {}", response);
/// }
///
/// ws.close().await?;
/// # Ok(())
/// # }
/// ```
#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
pub struct NativeWebSocket {
    connection: Option<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    is_connected: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
impl NativeWebSocket {
    /// Create a new NativeWebSocket instance
    ///
    /// # Examples
    /// ```rust,ignore
    /// use supabase::websocket::NativeWebSocket;
    ///
    /// let websocket = NativeWebSocket::new();
    /// ```
    pub fn new() -> Self {
        Self {
            connection: None,
            is_connected: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
#[async_trait::async_trait]
impl WebSocketConnection for NativeWebSocket {
    async fn connect(&mut self, url: &str) -> Result<()> {
        use std::sync::atomic::Ordering;
        use tokio_tungstenite::connect_async;

        tracing::debug!("Connecting to WebSocket: {}", url);

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| Error::network(format!("WebSocket connection failed: {}", e)))?;

        self.connection = Some(ws_stream);
        self.is_connected.store(true, Ordering::SeqCst);

        tracing::info!("Connected to WebSocket successfully");
        Ok(())
    }

    async fn send(&mut self, message: &str) -> Result<()> {
        use futures_util::SinkExt;
        use tokio_tungstenite::tungstenite::Message;

        if let Some(ref mut ws) = self.connection {
            ws.send(Message::Text(message.to_string()))
                .await
                .map_err(|e| Error::network(format!("Failed to send WebSocket message: {}", e)))?;

            tracing::debug!("Sent WebSocket message: {}", message);
            Ok(())
        } else {
            Err(Error::network("WebSocket not connected"))
        }
    }

    async fn receive(&mut self) -> Result<Option<String>> {
        use futures_util::StreamExt;
        use std::sync::atomic::Ordering;
        use tokio_tungstenite::tungstenite::Message;

        if let Some(ref mut ws) = self.connection {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    tracing::debug!("Received WebSocket message: {}", text);
                    Ok(Some(text))
                }
                Some(Ok(Message::Close(_))) => {
                    tracing::info!("WebSocket connection closed by remote");
                    self.is_connected.store(false, Ordering::SeqCst);
                    Ok(None)
                }
                Some(Err(e)) => {
                    tracing::error!("WebSocket error: {}", e);
                    self.is_connected.store(false, Ordering::SeqCst);
                    Err(Error::network(format!("WebSocket error: {}", e)))
                }
                None => Ok(None),
                _ => Ok(None), // Other message types (binary, ping, pong, etc.)
            }
        } else {
            Err(Error::network("WebSocket not connected"))
        }
    }

    async fn close(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;

        if let Some(ref mut ws) = self.connection {
            let _ = ws.close(None).await;
        }
        self.connection = None;
        self.is_connected.store(false, Ordering::SeqCst);

        tracing::info!("WebSocket connection closed");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.is_connected.load(Ordering::SeqCst)
    }
}

/// WASM WebSocket implementation using web-sys
///
/// This implementation provides WebSocket support for WASM targets using the
/// browser's native WebSocket API. Messages are queued internally and can be
/// retrieved using the `receive()` method.
///
/// ## Features
///
/// - Browser-native WebSocket support
/// - Automatic message queuing
/// - Event-driven architecture with callbacks
/// - Error handling through browser events
///
/// ## Example
///
/// ```rust,ignore
/// # #[cfg(target_arch = "wasm32")]
/// # async fn example() -> supabase::Result<()> {
/// use supabase::websocket::{WasmWebSocket, WebSocketConnection};
///
/// let mut ws = WasmWebSocket::new();
/// ws.connect("wss://echo.websocket.org").await?;
///
/// ws.send("Hello from WASM!").await?;
/// if let Some(response) = ws.receive().await? {
///     web_sys::console::log_1(&format!("Got: {}", response).into());
/// }
///
/// ws.close().await?;
/// # Ok(())
/// # }
/// ```
#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
pub struct WasmWebSocket {
    websocket: Option<web_sys::WebSocket>,
    is_connected: std::sync::Arc<std::sync::atomic::AtomicBool>,
    message_queue: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
}

#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
impl WasmWebSocket {
    /// Create a new WasmWebSocket instance
    ///
    /// # Examples
    /// ```rust,ignore
    /// # #[cfg(target_arch = "wasm32")]
    /// # {
    /// use supabase::websocket::WasmWebSocket;
    ///
    /// let websocket = WasmWebSocket::new();
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            websocket: None,
            is_connected: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            message_queue: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
        }
    }
}

#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
#[async_trait::async_trait(?Send)]
impl WebSocketConnection for WasmWebSocket {
    async fn connect(&mut self, url: &str) -> Result<()> {
        use std::sync::atomic::Ordering;
        use wasm_bindgen::{prelude::*, JsCast};
        use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

        web_sys::console::log_1(&format!("Connecting to WebSocket: {}", url).into());

        let websocket = WebSocket::new(url)
            .map_err(|e| Error::network(format!("Failed to create WebSocket: {:?}", e)))?;

        let is_connected = std::sync::Arc::clone(&self.is_connected);
        let message_queue = std::rc::Rc::clone(&self.message_queue);

        // Setup onopen callback
        let onopen_callback = {
            let is_connected = std::sync::Arc::clone(&is_connected);
            Closure::wrap(Box::new(move |_event: web_sys::Event| {
                web_sys::console::log_1(&"WebSocket connection opened".into());
                is_connected.store(true, Ordering::SeqCst);
            }) as Box<dyn FnMut(_)>)
        };
        websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // Setup onmessage callback
        let onmessage_callback = {
            let message_queue = std::rc::Rc::clone(&message_queue);
            Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
                    let message = String::from(text);
                    web_sys::console::log_1(
                        &format!("Received WebSocket message: {}", message).into(),
                    );
                    message_queue.borrow_mut().push(message);
                }
            }) as Box<dyn FnMut(_)>)
        };
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // Setup onerror callback
        let onerror_callback = {
            let is_connected = std::sync::Arc::clone(&is_connected);
            Closure::wrap(Box::new(move |event: ErrorEvent| {
                web_sys::console::log_1(&format!("WebSocket error: {:?}", event).into());
                is_connected.store(false, Ordering::SeqCst);
            }) as Box<dyn FnMut(_)>)
        };
        websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // Setup onclose callback
        let onclose_callback = {
            let is_connected = std::sync::Arc::clone(&is_connected);
            Closure::wrap(Box::new(move |event: CloseEvent| {
                web_sys::console::log_1(
                    &format!("WebSocket connection closed: {}", event.reason()).into(),
                );
                is_connected.store(false, Ordering::SeqCst);
            }) as Box<dyn FnMut(_)>)
        };
        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        self.websocket = Some(websocket);

        // Wait for connection to open (with timeout)
        let start_time = js_sys::Date::now();
        let timeout_ms = 5000.0; // 5 second timeout

        while !self.is_connected.load(Ordering::SeqCst)
            && (js_sys::Date::now() - start_time) < timeout_ms
        {
            // Simple delay using Promise
            let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
            wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .map_err(|e| Error::network(format!("Promise error: {:?}", e)))?;

            // Add a small delay
            let delay_promise = js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 10)
                    .unwrap();
            });
            wasm_bindgen_futures::JsFuture::from(delay_promise)
                .await
                .map_err(|e| Error::network(format!("Timeout error: {:?}", e)))?;
        }

        if !self.is_connected.load(Ordering::SeqCst) {
            return Err(Error::network("WebSocket connection timeout"));
        }

        web_sys::console::log_1(&"WebSocket connected successfully".into());
        Ok(())
    }

    async fn send(&mut self, message: &str) -> Result<()> {
        if let Some(ref websocket) = self.websocket {
            websocket.send_with_str(message).map_err(|e| {
                Error::network(format!("Failed to send WebSocket message: {:?}", e))
            })?;

            web_sys::console::log_1(&format!("Sent WebSocket message: {}", message).into());
            Ok(())
        } else {
            Err(Error::network("WebSocket not connected"))
        }
    }

    async fn receive(&mut self) -> Result<Option<String>> {
        let mut queue = self.message_queue.borrow_mut();
        if !queue.is_empty() {
            Ok(Some(queue.remove(0)))
        } else {
            Ok(None)
        }
    }

    async fn close(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;

        if let Some(ref websocket) = self.websocket {
            websocket.close().ok();
        }
        self.websocket = None;
        self.is_connected.store(false, Ordering::SeqCst);
        self.message_queue.borrow_mut().clear();

        web_sys::console::log_1(&"WebSocket connection closed".into());
        Ok(())
    }

    fn is_connected(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.is_connected.load(Ordering::SeqCst)
    }
}

/// Factory function to create appropriate WebSocket implementation
///
/// This function automatically creates the correct WebSocket implementation
/// based on the target platform:
///
/// - **Native targets**: Returns `NativeWebSocket`
/// - **WASM targets**: Returns `WasmWebSocket`
///
/// # Examples
///
/// ```rust,ignore
/// use supabase::websocket::{create_websocket, WebSocketConnection};
///
/// # async fn example() -> supabase::Result<()> {
/// // Automatically creates the right WebSocket for your platform
/// let mut ws = create_websocket();
/// ws.connect("wss://echo.websocket.org").await?;
///
/// // Works the same on both native and WASM
/// ws.send("Hello!").await?;
/// if let Some(msg) = ws.receive().await? {
///     println!("Received: {}", msg); // Native
///     // web_sys::console::log_1(&msg.into()); // WASM
/// }
///
/// ws.close().await?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "realtime")]
pub fn create_websocket() -> Box<dyn WebSocketConnection> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(NativeWebSocket::new())
    }

    #[cfg(target_arch = "wasm32")]
    {
        Box::new(WasmWebSocket::new())
    }
}

#[cfg(all(test, feature = "realtime"))]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_native_websocket_creation() {
        let ws = NativeWebSocket::new();
        assert!(!ws.is_connected());
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_wasm_websocket_creation() {
        let ws = WasmWebSocket::new();
        assert!(!ws.is_connected());
    }

    #[test]
    fn test_create_websocket() {
        let ws = create_websocket();
        assert!(!ws.is_connected());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_websocket_error_handling() {
        let mut ws = NativeWebSocket::new();

        // Should fail when not connected
        let result = ws.send("test").await;
        assert!(result.is_err());

        let result = ws.receive().await;
        assert!(result.is_err());
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn test_wasm_websocket_error_handling() {
        let mut ws = WasmWebSocket::new();

        // Should fail when not connected
        let result = ws.send("test").await;
        assert!(result.is_err());

        // Should return None when no messages
        let result = ws.receive().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_websocket_state_management() {
        let mut ws = NativeWebSocket::new();
        assert!(!ws.is_connected());

        // After close, should not be connected
        ws.close().await.unwrap();
        assert!(!ws.is_connected());
    }
}
