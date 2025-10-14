//! WASM-specific session management implementations
//!
//! This module provides browser-specific session management features including:
//! - BroadcastChannel for cross-tab communication
//! - Storage events for localStorage synchronization
//! - Browser-specific device/client detection

#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
use crate::error::{Error, Result};
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
use crate::session::{CrossTabChannel, CrossTabMessage};
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
use gloo_timers::callback::Timeout;
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
use std::cell::RefCell;
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
use std::rc::Rc;
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
use wasm_bindgen::prelude::*;
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
use wasm_bindgen::JsCast;
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
use web_sys::BroadcastChannel;

/// WASM cross-tab communication channel using BroadcastChannel API
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
pub struct WasmCrossTabChannel {
    channel: BroadcastChannel,
    _message_handler: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::MessageEvent)>>>>,
}

// WASM is single-threaded, so Send and Sync are safe to implement
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
unsafe impl Send for WasmCrossTabChannel {}
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
unsafe impl Sync for WasmCrossTabChannel {}

#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
impl WasmCrossTabChannel {
    /// Create a new cross-tab communication channel
    pub fn new() -> Result<Self> {
        let channel = BroadcastChannel::new("supabase_session_sync")
            .map_err(|_| Error::platform("Failed to create BroadcastChannel"))?;

        Ok(Self {
            channel,
            _message_handler: Rc::new(RefCell::new(None)),
        })
    }

    /// Create a channel with custom name
    pub fn new_with_name(channel_name: &str) -> Result<Self> {
        let channel = BroadcastChannel::new(channel_name)
            .map_err(|_| Error::platform("Failed to create BroadcastChannel"))?;

        Ok(Self {
            channel,
            _message_handler: Rc::new(RefCell::new(None)),
        })
    }
}

#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
#[async_trait::async_trait(?Send)]
impl CrossTabChannel for WasmCrossTabChannel {
    async fn send_message(&self, message: CrossTabMessage) -> Result<()> {
        let serialized = serde_json::to_string(&message).map_err(|e| {
            Error::platform(format!("Failed to serialize cross-tab message: {}", e))
        })?;

        let js_value = JsValue::from_str(&serialized);
        self.channel
            .post_message(&js_value)
            .map_err(|_| Error::platform("Failed to post message to BroadcastChannel"))?;

        Ok(())
    }

    fn on_message(&self, callback: Box<dyn Fn(CrossTabMessage) + Send + Sync>) {
        let callback = Rc::new(callback);
        let closure = Closure::wrap(Box::new({
            let callback = callback.clone();
            move |event: web_sys::MessageEvent| {
                if let Some(data) = event.data().as_string() {
                    if let Ok(message) = serde_json::from_str::<CrossTabMessage>(&data) {
                        callback(message);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        self.channel
            .set_onmessage(Some(closure.as_ref().unchecked_ref()));

        // Store closure to prevent it from being dropped
        *self._message_handler.borrow_mut() = Some(closure);
    }

    async fn close(&self) -> Result<()> {
        self.channel.close();
        Ok(())
    }
}

/// WASM utilities for device and client detection
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
pub struct WasmDeviceDetector;

#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
impl WasmDeviceDetector {
    /// Get browser information
    pub fn get_browser_info() -> Option<BrowserInfo> {
        let window = web_sys::window()?;
        let navigator = window.navigator();

        Some(BrowserInfo {
            user_agent: navigator.user_agent().ok(),
            platform: navigator.platform().ok(),
            language: navigator.language(),
            languages: navigator
                .languages()
                .iter()
                .filter_map(|v| v.as_string())
                .collect(),
            online: navigator.on_line(),
            cookie_enabled: true, // Assume cookies are enabled in WASM context
            do_not_track: Some(navigator.do_not_track()),
        })
    }

    /// Generate a client ID based on browser characteristics
    pub fn generate_client_id() -> Option<String> {
        let window = web_sys::window()?;
        let navigator = window.navigator();

        // Create a fingerprint based on available characteristics
        let mut fingerprint_data = Vec::new();

        if let Ok(user_agent) = navigator.user_agent() {
            fingerprint_data.push(user_agent);
        }

        if let Ok(platform) = navigator.platform() {
            fingerprint_data.push(platform);
        }

        if let Some(language) = navigator.language() {
            fingerprint_data.push(language);
        }

        // Try to access screen properties via js_sys if needed
        // Screen API might not be fully supported in all web_sys versions
        #[cfg(feature = "wasm")]
        if let Some(screen_width) = js_sys::Reflect::get(&window, &"screen".into())
            .ok()
            .and_then(|screen| js_sys::Reflect::get(&screen, &"width".into()).ok())
            .and_then(|w| w.as_f64())
        {
            fingerprint_data.push(screen_width.to_string());
        }

        // Create a hash of the fingerprint data
        let fingerprint = fingerprint_data.join("|");
        Some(format!("wasm_{}", simple_hash(&fingerprint)))
    }

    /// Generate a tab ID
    pub fn generate_tab_id() -> String {
        // Use performance.now() and random values for tab ID
        let performance_now = web_sys::window()
            .and_then(|w| {
                js_sys::Reflect::get(&w, &"performance".into())
                    .ok()
                    .and_then(|perf| {
                        let now_fn = js_sys::Reflect::get(&perf, &"now".into()).ok()?;
                        js_sys::Reflect::apply(&now_fn.into(), &perf, &js_sys::Array::new()).ok()
                    })
                    .and_then(|v| v.as_f64())
            })
            .unwrap_or(0.0);

        let random = js_sys::Math::random();
        format!(
            "tab_{}_{}",
            performance_now as u64,
            (random * 1000000.0) as u64
        )
    }

    /// Check if storage is available
    pub fn is_storage_available() -> bool {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .is_some()
    }

    /// Get storage quota information
    pub fn get_storage_info() -> Option<StorageInfo> {
        let window = web_sys::window()?;
        let _navigator = window.navigator();

        // Note: Storage API is not fully supported in all browsers
        // This is a simplified version
        Some(StorageInfo {
            available: Self::is_storage_available(),
            estimated_quota: None, // Would need Storage API
            estimated_usage: None, // Would need Storage API
        })
    }
}

/// Browser information structure
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
#[derive(Debug, Clone)]
pub struct BrowserInfo {
    pub user_agent: Option<String>,
    pub platform: Option<String>,
    pub language: Option<String>,
    pub languages: Vec<String>,
    pub online: bool,
    pub cookie_enabled: bool,
    pub do_not_track: Option<String>,
}

/// Storage information structure
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub available: bool,
    pub estimated_quota: Option<u64>,
    pub estimated_usage: Option<u64>,
}

/// WASM session monitor for tracking browser events
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
pub struct WasmSessionMonitor {
    visibility_handler: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>>,
    focus_handler: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>>,
    blur_handler: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>>,
    beforeunload_handler: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>>,
}

#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
impl WasmSessionMonitor {
    pub fn new() -> Self {
        Self {
            visibility_handler: Rc::new(RefCell::new(None)),
            focus_handler: Rc::new(RefCell::new(None)),
            blur_handler: Rc::new(RefCell::new(None)),
            beforeunload_handler: Rc::new(RefCell::new(None)),
        }
    }

    /// Start monitoring session events
    pub fn start_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(SessionMonitorEvent) + 'static,
    {
        let window =
            web_sys::window().ok_or_else(|| Error::platform("No window object available"))?;

        let document = window
            .document()
            .ok_or_else(|| Error::platform("No document object available"))?;

        let callback = Rc::new(callback);

        // Visibility change handler
        {
            let callback = callback.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let visible = web_sys::window()
                    .and_then(|w| w.document())
                    .map(|d| !d.hidden())
                    .unwrap_or(false);

                if visible {
                    callback(SessionMonitorEvent::TabVisible);
                } else {
                    callback(SessionMonitorEvent::TabHidden);
                }
            }) as Box<dyn FnMut(_)>);

            document
                .add_event_listener_with_callback(
                    "visibilitychange",
                    closure.as_ref().unchecked_ref(),
                )
                .map_err(|_| Error::platform("Failed to add visibility change listener"))?;

            *self.visibility_handler.borrow_mut() = Some(closure);
        }

        // Focus handler
        {
            let callback = callback.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                callback(SessionMonitorEvent::WindowFocused);
            }) as Box<dyn FnMut(_)>);

            window
                .add_event_listener_with_callback("focus", closure.as_ref().unchecked_ref())
                .map_err(|_| Error::platform("Failed to add focus listener"))?;

            *self.focus_handler.borrow_mut() = Some(closure);
        }

        // Blur handler
        {
            let callback = callback.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                callback(SessionMonitorEvent::WindowBlurred);
            }) as Box<dyn FnMut(_)>);

            window
                .add_event_listener_with_callback("blur", closure.as_ref().unchecked_ref())
                .map_err(|_| Error::platform("Failed to add blur listener"))?;

            *self.blur_handler.borrow_mut() = Some(closure);
        }

        // Before unload handler
        {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                callback(SessionMonitorEvent::BeforeUnload);
            }) as Box<dyn FnMut(_)>);

            window
                .add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref())
                .map_err(|_| Error::platform("Failed to add beforeunload listener"))?;

            *self.beforeunload_handler.borrow_mut() = Some(closure);
        }

        Ok(())
    }
}

/// Session monitor events
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
#[derive(Debug, Clone)]
pub enum SessionMonitorEvent {
    TabVisible,
    TabHidden,
    WindowFocused,
    WindowBlurred,
    BeforeUnload,
}

/// Simple hash function for fingerprinting
#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
fn simple_hash(input: &str) -> u64 {
    let mut hash = 0u64;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

#[cfg(all(
    feature = "session-management",
    target_arch = "wasm32",
    feature = "wasm"
))]
impl Default for WasmSessionMonitor {
    fn default() -> Self {
        Self::new()
    }
}
