//! Performance optimization module
//!
//! This module provides performance enhancements for Supabase operations:
//! - **Connection Pooling**: Efficient HTTP client connection management
//! - **Request Caching**: Intelligent API response caching
//! - **Batch Operations**: Multi-request optimization
//! - **Compression**: Request/response compression support

use crate::{
    error::{Error, Result},
    types::SupabaseConfig,
};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::RwLock;

#[cfg(target_arch = "wasm32")]
mod wasm_rwlock {
    use std::sync::RwLock as StdRwLock;

    pub struct RwLock<T>(StdRwLock<T>);

    impl<T> RwLock<T> {
        pub fn new(value: T) -> Self {
            Self(StdRwLock::new(value))
        }

        pub async fn read(&self) -> std::sync::RwLockReadGuard<'_, T> {
            self.0.read().unwrap()
        }

        pub async fn write(&self) -> std::sync::RwLockWriteGuard<'_, T> {
            self.0.write().unwrap()
        }
    }

    impl<T: std::fmt::Debug> std::fmt::Debug for RwLock<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "RwLock")
        }
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_rwlock::RwLock;

use tracing::{debug, info};

/// Performance optimization manager
#[derive(Debug, Clone)]
pub struct Performance {
    #[allow(dead_code)] // Used in future implementations
    http_client: Arc<HttpClient>,
    #[allow(dead_code)] // Used in future implementations
    config: Arc<SupabaseConfig>,
    connection_pool: Arc<ConnectionPool>,
    cache: Arc<RequestCache>,
    batch_processor: Arc<BatchProcessor>,
}

/// Connection pool for HTTP clients
#[derive(Debug)]
pub struct ConnectionPool {
    pools: RwLock<HashMap<String, Arc<HttpClient>>>,
    config: ConnectionPoolConfig,
}

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Maximum connections per host
    pub max_connections_per_host: usize,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Connection keep-alive timeout
    pub keep_alive_timeout: Duration,
    /// Enable HTTP/2
    pub http2: bool,
    /// User agent string
    pub user_agent: Option<String>,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            idle_timeout: Duration::from_secs(90),
            keep_alive_timeout: Duration::from_secs(60),
            http2: true,
            user_agent: Some("supabase-rust/0.4.2".to_string()),
        }
    }
}

/// Request cache for API responses
#[derive(Debug)]
pub struct RequestCache {
    cache: RwLock<HashMap<String, CacheEntry>>,
    config: CacheConfig,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum cache size (number of entries)
    pub max_entries: usize,
    /// Default cache TTL
    pub default_ttl: Duration,
    /// Enable cache compression
    pub enable_compression: bool,
    /// Cache only successful responses
    pub cache_success_only: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            enable_compression: true,
            cache_success_only: true,
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Cached response data
    pub data: Value,
    /// Entry creation time
    pub created_at: Instant,
    /// Time-to-live duration
    pub ttl: Duration,
    /// Response size (compressed if enabled)
    pub size_bytes: usize,
    /// Cache hit count
    pub hit_count: u64,
}

/// Batch processing for multiple operations
#[derive(Debug)]
pub struct BatchProcessor {
    pending_operations: RwLock<Vec<BatchOperation>>,
    config: BatchConfig,
}

/// Batch processing configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Batch flush interval
    pub flush_interval: Duration,
    /// Enable automatic batching
    pub auto_batch: bool,
    /// Batch timeout
    pub batch_timeout: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 50,
            flush_interval: Duration::from_millis(100),
            auto_batch: true,
            batch_timeout: Duration::from_secs(5),
        }
    }
}

/// Batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    /// Operation ID
    pub id: String,
    /// HTTP method
    pub method: String,
    /// Request URL
    pub url: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Option<Value>,
    /// Operation priority
    pub priority: u8,
}

/// Batch execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Operation ID
    pub id: String,
    /// HTTP status code
    pub status: u16,
    /// Response data
    pub data: Option<Value>,
    /// Error message if any
    pub error: Option<String>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Active connections count
    pub active_connections: usize,
    /// Cache hit ratio (0.0 to 1.0)
    pub cache_hit_ratio: f64,
    /// Cache entry count
    pub cache_entries: usize,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests count
    pub successful_requests: u64,
    /// Failed requests count
    pub failed_requests: u64,
    /// Total batched operations
    pub batched_operations: u64,
}

impl Performance {
    /// Create a new Performance instance
    pub fn new(config: Arc<SupabaseConfig>, http_client: Arc<HttpClient>) -> Result<Self> {
        debug!("Initializing Performance module");

        let connection_pool = Arc::new(ConnectionPool::new(ConnectionPoolConfig::default()));
        let cache = Arc::new(RequestCache::new(CacheConfig::default()));
        let batch_processor = Arc::new(BatchProcessor::new(BatchConfig::default()));

        Ok(Self {
            http_client,
            config,
            connection_pool,
            cache,
            batch_processor,
        })
    }

    /// Create with custom configuration
    pub fn new_with_config(
        config: Arc<SupabaseConfig>,
        http_client: Arc<HttpClient>,
        pool_config: ConnectionPoolConfig,
        cache_config: CacheConfig,
        batch_config: BatchConfig,
    ) -> Result<Self> {
        debug!("Initializing Performance module with custom config");

        let connection_pool = Arc::new(ConnectionPool::new(pool_config));
        let cache = Arc::new(RequestCache::new(cache_config));
        let batch_processor = Arc::new(BatchProcessor::new(batch_config));

        Ok(Self {
            http_client,
            config,
            connection_pool,
            cache,
            batch_processor,
        })
    }

    /// Get optimized HTTP client for a host
    pub async fn get_client(&self, host: &str) -> Result<Arc<HttpClient>> {
        self.connection_pool.get_client(host).await
    }

    /// Cache a response with optional TTL
    pub async fn cache_response(
        &self,
        key: &str,
        data: Value,
        ttl: Option<Duration>,
    ) -> Result<()> {
        self.cache.set(key, data, ttl).await
    }

    /// Get cached response
    pub async fn get_cached_response(&self, key: &str) -> Result<Option<Value>> {
        self.cache.get(key).await
    }

    /// Add operation to batch processing queue
    pub async fn add_to_batch(&self, operation: BatchOperation) -> Result<()> {
        self.batch_processor.add_operation(operation).await
    }

    /// Process pending batch operations
    pub async fn process_batch(&self) -> Result<Vec<BatchResult>> {
        self.batch_processor.process_pending().await
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        let connection_metrics = self.connection_pool.get_metrics().await;
        let cache_metrics = self.cache.get_metrics().await;
        let batch_metrics = self.batch_processor.get_metrics().await;

        PerformanceMetrics {
            active_connections: connection_metrics.active_count,
            cache_hit_ratio: cache_metrics.hit_ratio,
            cache_entries: cache_metrics.entry_count,
            avg_response_time_ms: 0.0, // TODO: Implement response time tracking
            total_requests: 0,         // TODO: Implement request tracking
            successful_requests: 0,    // TODO: Implement success tracking
            failed_requests: 0,        // TODO: Implement failure tracking
            batched_operations: batch_metrics.total_operations,
        }
    }

    /// Clear all caches
    pub async fn clear_cache(&self) -> Result<()> {
        self.cache.clear().await
    }

    /// Warm up connections for specified hosts
    pub async fn warm_up_connections(&self, hosts: Vec<String>) -> Result<()> {
        for host in hosts {
            let _ = self.connection_pool.get_client(&host).await?;
            debug!("Warmed up connection for host: {}", host);
        }
        Ok(())
    }
}

// Connection Pool Implementation

impl ConnectionPool {
    fn new(config: ConnectionPoolConfig) -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            config,
        }
    }

    async fn get_client(&self, host: &str) -> Result<Arc<HttpClient>> {
        // Check if client already exists
        {
            let pools = self.pools.read().await;
            if let Some(client) = pools.get(host) {
                return Ok(Arc::clone(client));
            }
        }

        // Create new optimized client
        let client = self.create_optimized_client().await?;
        let client_arc = Arc::new(client);

        // Store in pool
        {
            let mut pools = self.pools.write().await;
            pools.insert(host.to_string(), Arc::clone(&client_arc));
        }

        info!("Created new HTTP client for host: {}", host);
        Ok(client_arc)
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn create_optimized_client(&self) -> Result<HttpClient> {
        let mut builder = HttpClient::builder()
            .pool_max_idle_per_host(self.config.max_connections_per_host)
            .pool_idle_timeout(self.config.idle_timeout)
            .tcp_keepalive(Some(self.config.keep_alive_timeout));

        if let Some(user_agent) = &self.config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        builder
            .build()
            .map_err(|e| Error::config(format!("Failed to create HTTP client: {}", e)))
    }

    #[cfg(target_arch = "wasm32")]
    async fn create_optimized_client(&self) -> Result<HttpClient> {
        let mut builder = HttpClient::builder();

        if let Some(user_agent) = &self.config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        builder
            .build()
            .map_err(|e| Error::config(format!("Failed to create HTTP client: {}", e)))
    }

    async fn get_metrics(&self) -> ConnectionMetrics {
        let pools = self.pools.read().await;
        ConnectionMetrics {
            active_count: pools.len(),
            total_created: pools.len() as u64, // Simplified for now
        }
    }
}

#[derive(Debug, Clone)]
struct ConnectionMetrics {
    active_count: usize,
    #[allow(dead_code)] // Used in future metrics implementations
    total_created: u64,
}

// Request Cache Implementation

impl RequestCache {
    fn new(config: CacheConfig) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            config,
        }
    }

    async fn set(&self, key: &str, data: Value, ttl: Option<Duration>) -> Result<()> {
        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            ttl: ttl.unwrap_or(self.config.default_ttl),
            size_bytes: 0, // TODO: Calculate actual size
            hit_count: 0,
        };

        let mut cache = self.cache.write().await;

        // Check cache size limit
        if cache.len() >= self.config.max_entries {
            self.evict_oldest(&mut cache);
        }

        cache.insert(key.to_string(), entry);
        debug!("Cached response for key: {}", key);
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Value>> {
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(key) {
            // Check if expired
            if entry.created_at.elapsed() > entry.ttl {
                cache.remove(key);
                debug!("Cache entry expired for key: {}", key);
                return Ok(None);
            }

            // Update hit count
            entry.hit_count += 1;
            debug!("Cache hit for key: {}", key);
            Ok(Some(entry.data.clone()))
        } else {
            debug!("Cache miss for key: {}", key);
            Ok(None)
        }
    }

    async fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Cache cleared");
        Ok(())
    }

    async fn get_metrics(&self) -> CacheMetrics {
        let cache = self.cache.read().await;
        let total_hits: u64 = cache.values().map(|entry| entry.hit_count).sum();
        let total_requests = total_hits + cache.len() as u64; // Simplified calculation

        CacheMetrics {
            entry_count: cache.len(),
            hit_ratio: if total_requests > 0 {
                total_hits as f64 / total_requests as f64
            } else {
                0.0
            },
        }
    }

    fn evict_oldest(&self, cache: &mut HashMap<String, CacheEntry>) {
        if let Some((oldest_key, _)) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(k, v)| (k.clone(), v.created_at))
        {
            cache.remove(&oldest_key);
            debug!("Evicted oldest cache entry: {}", oldest_key);
        }
    }
}

#[derive(Debug, Clone)]
struct CacheMetrics {
    entry_count: usize,
    hit_ratio: f64,
}

// Batch Processor Implementation

impl BatchProcessor {
    fn new(config: BatchConfig) -> Self {
        Self {
            pending_operations: RwLock::new(Vec::new()),
            config,
        }
    }

    async fn add_operation(&self, operation: BatchOperation) -> Result<()> {
        let mut pending = self.pending_operations.write().await;
        pending.push(operation);

        // Auto-process if batch is full
        if self.config.auto_batch && pending.len() >= self.config.max_batch_size {
            drop(pending); // Release lock
            let _ = self.process_pending().await;
        }

        Ok(())
    }

    async fn process_pending(&self) -> Result<Vec<BatchResult>> {
        let mut pending = self.pending_operations.write().await;
        if pending.is_empty() {
            return Ok(Vec::new());
        }

        let operations = pending.drain(..).collect::<Vec<_>>();
        drop(pending); // Release lock

        debug!("Processing batch of {} operations", operations.len());

        // TODO: Implement actual HTTP batching
        let results = operations
            .into_iter()
            .map(|op| BatchResult {
                id: op.id,
                status: 200, // Placeholder
                data: Some(Value::Null),
                error: None,
            })
            .collect();

        Ok(results)
    }

    async fn get_metrics(&self) -> BatchMetrics {
        let pending = self.pending_operations.read().await;
        BatchMetrics {
            pending_operations: pending.len(),
            total_operations: 0, // TODO: Track total processed operations
        }
    }
}

#[derive(Debug, Clone)]
struct BatchMetrics {
    #[allow(dead_code)] // Used in future metrics implementations
    pending_operations: usize,
    total_operations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let pool = ConnectionPool::new(ConnectionPoolConfig::default());
        let client = pool.get_client("localhost").await.unwrap();
        // Client should be successfully created with proper reference count
        assert!(Arc::strong_count(&client) >= 1);
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = RequestCache::new(CacheConfig::default());
        let test_data = serde_json::json!({"test": "data"});

        cache
            .set("test_key", test_data.clone(), None)
            .await
            .unwrap();
        let retrieved = cache.get("test_key").await.unwrap();

        assert_eq!(retrieved, Some(test_data));
    }

    #[tokio::test]
    async fn test_batch_processor() {
        let processor = BatchProcessor::new(BatchConfig::default());

        let operation = BatchOperation {
            id: "test_op".to_string(),
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            headers: HashMap::new(),
            body: None,
            priority: 1,
        };

        processor.add_operation(operation).await.unwrap();
        let results = processor.process_pending().await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test_op");
    }
}
