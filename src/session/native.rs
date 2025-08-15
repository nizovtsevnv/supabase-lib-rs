//! Native-specific session management implementations
//!
//! This module provides desktop/server-specific session management features including:
//! - File-based cross-process communication
//! - System-level device detection
//! - Native process monitoring

// Type alias for complex callback type
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
type MessageCallback = Box<dyn Fn(CrossTabMessage) + Send + Sync>;

#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
use crate::error::{Error, Result};
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
use crate::session::{CrossTabChannel, CrossTabMessage};
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
use std::path::PathBuf;
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
use std::sync::Arc;
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
use tokio::sync::Mutex;
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
use tokio::time::interval;

/// Native cross-process communication channel using file system
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
pub struct NativeCrossTabChannel {
    channel_dir: PathBuf,
    process_id: String,
    message_callbacks: Arc<Mutex<Vec<MessageCallback>>>,
    _monitor_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
impl NativeCrossTabChannel {
    /// Create a new cross-process communication channel
    pub fn new() -> Result<Self> {
        let channel_dir = dirs::cache_dir()
            .ok_or_else(|| Error::platform("Could not determine cache directory"))?
            .join("supabase")
            .join("session_channel");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&channel_dir)
            .map_err(|e| Error::platform(format!("Failed to create channel directory: {}", e)))?;

        let process_id = format!(
            "{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        let channel = Self {
            channel_dir,
            process_id,
            message_callbacks: Arc::new(Mutex::new(Vec::new())),
            _monitor_handle: Arc::new(Mutex::new(None)),
        };

        // Start monitoring for incoming messages
        channel.start_monitoring()?;

        Ok(channel)
    }

    /// Create a channel with custom directory
    pub fn new_with_dir(channel_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&channel_dir)
            .map_err(|e| Error::platform(format!("Failed to create channel directory: {}", e)))?;

        let process_id = format!(
            "{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        let channel = Self {
            channel_dir,
            process_id,
            message_callbacks: Arc::new(Mutex::new(Vec::new())),
            _monitor_handle: Arc::new(Mutex::new(None)),
        };

        channel.start_monitoring()?;
        Ok(channel)
    }

    fn get_message_file_path(&self, message_id: &str) -> PathBuf {
        self.channel_dir.join(format!("{}.json", message_id))
    }

    fn start_monitoring(&self) -> Result<()> {
        let channel_dir = self.channel_dir.clone();
        let process_id = self.process_id.clone();
        let callbacks = self.message_callbacks.clone();

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100));
            let mut seen_messages = std::collections::HashSet::new();

            loop {
                interval.tick().await;

                // Read directory for new messages
                if let Ok(mut entries) = tokio::fs::read_dir(&channel_dir).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();

                        if path.extension().and_then(|s| s.to_str()) == Some("json") {
                            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                                if !seen_messages.contains(file_name) {
                                    seen_messages.insert(file_name.to_string());

                                    // Try to read and parse the message
                                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                                        if let Ok(message) =
                                            serde_json::from_str::<CrossTabMessage>(&content)
                                        {
                                            // Don't process our own messages
                                            if message.source_tab != process_id {
                                                let callbacks = callbacks.lock().await;
                                                for callback in callbacks.iter() {
                                                    callback(message.clone());
                                                }
                                            }
                                        }
                                    }

                                    // Clean up old message file
                                    let _ = tokio::fs::remove_file(&path).await;
                                }
                            }
                        }
                    }
                }

                // Clean up old seen messages (prevent memory leak)
                if seen_messages.len() > 1000 {
                    seen_messages.clear();
                }
            }
        });

        let mut monitor_handle = futures::executor::block_on(self._monitor_handle.lock());
        *monitor_handle = Some(handle);

        Ok(())
    }
}

#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
#[async_trait::async_trait]
impl CrossTabChannel for NativeCrossTabChannel {
    async fn send_message(&self, message: CrossTabMessage) -> Result<()> {
        let file_path = self.get_message_file_path(&message.message_id.to_string());
        let serialized = serde_json::to_string_pretty(&message).map_err(|e| {
            Error::platform(format!("Failed to serialize cross-tab message: {}", e))
        })?;

        tokio::fs::write(&file_path, serialized)
            .await
            .map_err(|e| Error::platform(format!("Failed to write message file: {}", e)))?;

        Ok(())
    }

    fn on_message(&self, callback: Box<dyn Fn(CrossTabMessage) + Send + Sync>) {
        let callbacks = self.message_callbacks.clone();
        tokio::spawn(async move {
            let mut callbacks = callbacks.lock().await;
            callbacks.push(callback);
        });
    }

    async fn close(&self) -> Result<()> {
        // Stop monitoring
        let mut monitor_handle = self._monitor_handle.lock().await;
        if let Some(handle) = monitor_handle.take() {
            handle.abort();
        }

        // Clean up any remaining message files from this process
        if let Ok(mut entries) = tokio::fs::read_dir(&self.channel_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    // Read the message to check if it's from this process
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if let Ok(message) = serde_json::from_str::<CrossTabMessage>(&content) {
                            if message.source_tab == self.process_id {
                                let _ = tokio::fs::remove_file(&path).await;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Native utilities for system and device detection
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
pub struct NativeDeviceDetector;

#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
impl NativeDeviceDetector {
    /// Get system information
    pub fn get_system_info() -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            family: std::env::consts::FAMILY.to_string(),
            hostname: hostname::get().ok().and_then(|h| h.into_string().ok()),
            username: std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .ok(),
            process_id: std::process::id(),
            executable_path: std::env::current_exe().ok(),
        }
    }

    /// Generate a device ID based on system characteristics
    pub fn generate_device_id() -> Result<String> {
        let mut device_data = Vec::new();

        // Operating system and architecture
        device_data.push(std::env::consts::OS.to_string());
        device_data.push(std::env::consts::ARCH.to_string());

        // Hostname
        if let Ok(hostname) = hostname::get() {
            if let Ok(hostname_str) = hostname.into_string() {
                device_data.push(hostname_str);
            }
        }

        // MAC address (if available)
        if let Ok(Some(mac)) = mac_address::get_mac_address() {
            device_data.push(mac.to_string());
        }

        // CPU info (simplified)
        device_data.push(num_cpus::get().to_string());

        // Create a hash of the device data
        let device_fingerprint = device_data.join("|");
        Ok(format!("native_{}", simple_hash(&device_fingerprint)))
    }

    /// Generate a process/session ID
    pub fn generate_session_id() -> String {
        format!(
            "proc_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        )
    }

    /// Get memory information
    pub fn get_memory_info() -> Option<MemoryInfo> {
        // This would require platform-specific implementations
        // For now, return a basic structure
        Some(MemoryInfo {
            total_physical: None,
            available_physical: None,
            total_virtual: None,
            available_virtual: None,
        })
    }

    /// Get disk space information
    pub fn get_disk_info() -> Option<DiskInfo> {
        // This would require platform-specific implementations
        Some(DiskInfo {
            total_space: None,
            available_space: None,
            used_space: None,
        })
    }

    /// Check if running in a container
    pub fn is_containerized() -> bool {
        // Check for common container indicators
        std::path::Path::new("/.dockerenv").exists()
            || std::env::var("container").is_ok()
            || std::fs::read_to_string("/proc/1/cgroup")
                .map(|content| content.contains("docker") || content.contains("lxc"))
                .unwrap_or(false)
    }

    /// Get environment type
    pub fn get_environment_type() -> EnvironmentType {
        if Self::is_containerized() {
            EnvironmentType::Container
        } else if std::env::var("SSH_CLIENT").is_ok() || std::env::var("SSH_TTY").is_ok() {
            EnvironmentType::RemoteSession
        } else {
            EnvironmentType::Local
        }
    }
}

/// System information structure
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub family: String,
    pub hostname: Option<String>,
    pub username: Option<String>,
    pub process_id: u32,
    pub executable_path: Option<PathBuf>,
}

/// Memory information structure
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_physical: Option<u64>,
    pub available_physical: Option<u64>,
    pub total_virtual: Option<u64>,
    pub available_virtual: Option<u64>,
}

/// Disk information structure
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub total_space: Option<u64>,
    pub available_space: Option<u64>,
    pub used_space: Option<u64>,
}

/// Environment type enumeration
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentType {
    Local,
    RemoteSession,
    Container,
}

/// Native session monitor for tracking system events
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
pub struct NativeSessionMonitor {
    monitoring: Arc<Mutex<bool>>,
    _monitor_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
impl NativeSessionMonitor {
    pub fn new() -> Self {
        Self {
            monitoring: Arc::new(Mutex::new(false)),
            _monitor_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Start monitoring system events
    pub async fn start_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(SessionMonitorEvent) + Send + Sync + 'static,
    {
        let mut monitoring = self.monitoring.lock().await;
        if *monitoring {
            return Ok(()); // Already monitoring
        }
        *monitoring = true;

        let callback = Arc::new(callback);
        let monitoring_flag = self.monitoring.clone();

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            let mut last_check = Instant::now();

            while *monitoring_flag.lock().await {
                interval.tick().await;

                let now = Instant::now();

                // Check for system changes (simplified)
                if now.duration_since(last_check) > Duration::from_secs(60) {
                    callback(SessionMonitorEvent::SystemCheck);
                    last_check = now;
                }

                // Check memory pressure (simplified)
                if let Some(_memory_info) = NativeDeviceDetector::get_memory_info() {
                    // Would implement actual memory pressure detection here
                    // callback(SessionMonitorEvent::MemoryPressure);
                }

                // Check disk space (simplified)
                if let Some(_disk_info) = NativeDeviceDetector::get_disk_info() {
                    // Would implement actual disk space monitoring here
                    // callback(SessionMonitorEvent::LowDiskSpace);
                }
            }
        });

        let mut monitor_handle = self._monitor_handle.lock().await;
        *monitor_handle = Some(handle);

        Ok(())
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        let mut monitoring = self.monitoring.lock().await;
        *monitoring = false;

        let mut monitor_handle = self._monitor_handle.lock().await;
        if let Some(handle) = monitor_handle.take() {
            handle.abort();
        }

        Ok(())
    }
}

/// Session monitor events
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
#[derive(Debug, Clone)]
pub enum SessionMonitorEvent {
    SystemCheck,
    MemoryPressure,
    LowDiskSpace,
    NetworkChange,
    ProcessTermination,
}

/// Simple hash function for fingerprinting
#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
fn simple_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

#[cfg(all(feature = "session-management", not(target_arch = "wasm32")))]
impl Default for NativeSessionMonitor {
    fn default() -> Self {
        Self::new()
    }
}
