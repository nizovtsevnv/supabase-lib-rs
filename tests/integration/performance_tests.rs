//! Integration tests for Performance optimization features (v0.4.2)

use std::{collections::HashMap, time::Duration};
use supabase::performance::{
    BatchConfig, BatchOperation, CacheConfig, ConnectionPoolConfig, PerformanceMetrics,
};

#[tokio::test]
async fn test_connection_pool_config() {
    let config = ConnectionPoolConfig {
        max_connections_per_host: 25,
        idle_timeout: Duration::from_secs(180),
        keep_alive_timeout: Duration::from_secs(120),
        http2: true,
        user_agent: Some("test-client/1.0".to_string()),
    };

    assert_eq!(config.max_connections_per_host, 25);
    assert_eq!(config.idle_timeout, Duration::from_secs(180));
    assert_eq!(config.keep_alive_timeout, Duration::from_secs(120));
    assert!(config.http2);
    assert_eq!(config.user_agent, Some("test-client/1.0".to_string()));
}

#[tokio::test]
async fn test_connection_pool_config_default() {
    let config = ConnectionPoolConfig::default();

    assert_eq!(config.max_connections_per_host, 10);
    assert_eq!(config.idle_timeout, Duration::from_secs(90));
    assert_eq!(config.keep_alive_timeout, Duration::from_secs(60));
    assert!(config.http2);
    assert_eq!(config.user_agent, Some("supabase-rust/0.4.2".to_string()));
}

#[tokio::test]
async fn test_cache_config() {
    let config = CacheConfig {
        max_entries: 2000,
        default_ttl: Duration::from_secs(900),
        enable_compression: false,
        cache_success_only: false,
    };

    assert_eq!(config.max_entries, 2000);
    assert_eq!(config.default_ttl, Duration::from_secs(900));
    assert!(!config.enable_compression);
    assert!(!config.cache_success_only);
}

#[tokio::test]
async fn test_cache_config_default() {
    let config = CacheConfig::default();

    assert_eq!(config.max_entries, 1000);
    assert_eq!(config.default_ttl, Duration::from_secs(300));
    assert!(config.enable_compression);
    assert!(config.cache_success_only);
}

#[tokio::test]
async fn test_batch_config() {
    let config = BatchConfig {
        max_batch_size: 100,
        flush_interval: Duration::from_millis(200),
        auto_batch: false,
        batch_timeout: Duration::from_secs(10),
    };

    assert_eq!(config.max_batch_size, 100);
    assert_eq!(config.flush_interval, Duration::from_millis(200));
    assert!(!config.auto_batch);
    assert_eq!(config.batch_timeout, Duration::from_secs(10));
}

#[tokio::test]
async fn test_batch_config_default() {
    let config = BatchConfig::default();

    assert_eq!(config.max_batch_size, 50);
    assert_eq!(config.flush_interval, Duration::from_millis(100));
    assert!(config.auto_batch);
    assert_eq!(config.batch_timeout, Duration::from_secs(5));
}

#[tokio::test]
async fn test_batch_operation_creation() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let operation = BatchOperation {
        id: "batch_op_1".to_string(),
        method: "POST".to_string(),
        url: "https://api.supabase.co/rest/v1/users".to_string(),
        headers,
        body: Some(serde_json::json!({
            "name": "John Doe",
            "email": "john@example.com"
        })),
        priority: 1,
    };

    assert_eq!(operation.id, "batch_op_1");
    assert_eq!(operation.method, "POST");
    assert_eq!(operation.priority, 1);
    assert!(operation.body.is_some());
    assert!(operation.headers.contains_key("Authorization"));
}

#[tokio::test]
async fn test_batch_operation_serialization() {
    let operation = BatchOperation {
        id: "test_op".to_string(),
        method: "GET".to_string(),
        url: "https://example.com/api".to_string(),
        headers: HashMap::new(),
        body: None,
        priority: 0,
    };

    // Test JSON serialization
    let json = serde_json::to_string(&operation).unwrap();
    let deserialized: BatchOperation = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "test_op");
    assert_eq!(deserialized.method, "GET");
    assert_eq!(deserialized.url, "https://example.com/api");
    assert_eq!(deserialized.priority, 0);
    assert!(deserialized.body.is_none());
}

#[tokio::test]
async fn test_performance_metrics_structure() {
    let metrics = PerformanceMetrics {
        active_connections: 5,
        cache_hit_ratio: 0.85,
        cache_entries: 150,
        avg_response_time_ms: 45.7,
        total_requests: 1000,
        successful_requests: 950,
        failed_requests: 50,
        batched_operations: 25,
    };

    assert_eq!(metrics.active_connections, 5);
    assert_eq!(metrics.cache_hit_ratio, 0.85);
    assert_eq!(metrics.cache_entries, 150);
    assert_eq!(metrics.avg_response_time_ms, 45.7);
    assert_eq!(metrics.total_requests, 1000);
    assert_eq!(metrics.successful_requests, 950);
    assert_eq!(metrics.failed_requests, 50);
    assert_eq!(metrics.batched_operations, 25);

    // Test that success + failed = total
    assert_eq!(metrics.successful_requests + metrics.failed_requests, metrics.total_requests);
}

#[tokio::test]
async fn test_performance_metrics_serialization() {
    let metrics = PerformanceMetrics {
        active_connections: 3,
        cache_hit_ratio: 0.72,
        cache_entries: 200,
        avg_response_time_ms: 35.2,
        total_requests: 500,
        successful_requests: 485,
        failed_requests: 15,
        batched_operations: 12,
    };

    // Test JSON serialization
    let json = serde_json::to_string(&metrics).unwrap();
    let deserialized: PerformanceMetrics = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.active_connections, 3);
    assert_eq!(deserialized.cache_hit_ratio, 0.72);
    assert_eq!(deserialized.avg_response_time_ms, 35.2);
}

// Integration tests that would require actual HTTP clients and connections
#[tokio::test]
#[ignore = "Requires HTTP client integration"]
async fn test_connection_pool_integration() {
    use supabase::performance::ConnectionPool;

    let config = ConnectionPoolConfig::default();
    let pool = ConnectionPool::new(config);

    // In a real test, we would:
    // 1. Get a client from the pool
    // 2. Make HTTP requests
    // 3. Verify connection reuse
    // 4. Test connection limits
}

#[tokio::test]
#[ignore = "Requires cache storage integration"]
async fn test_request_cache_integration() {
    use supabase::performance::RequestCache;

    let config = CacheConfig::default();
    let cache = RequestCache::new(config);

    // In a real test, we would:
    // 1. Cache some responses
    // 2. Retrieve them and verify hit/miss
    // 3. Test TTL expiration
    // 4. Test cache eviction policies
}

#[tokio::test]
async fn test_batch_operations_priority_sorting() {
    let operations = vec![
        BatchOperation {
            id: "low_priority".to_string(),
            method: "GET".to_string(),
            url: "https://example.com/low".to_string(),
            headers: HashMap::new(),
            body: None,
            priority: 3,
        },
        BatchOperation {
            id: "high_priority".to_string(),
            method: "POST".to_string(),
            url: "https://example.com/high".to_string(),
            headers: HashMap::new(),
            body: Some(serde_json::json!({"urgent": true})),
            priority: 1,
        },
        BatchOperation {
            id: "medium_priority".to_string(),
            method: "PUT".to_string(),
            url: "https://example.com/medium".to_string(),
            headers: HashMap::new(),
            body: None,
            priority: 2,
        },
    ];

    // Sort by priority (lower numbers = higher priority)
    let mut sorted_ops = operations;
    sorted_ops.sort_by_key(|op| op.priority);

    assert_eq!(sorted_ops[0].id, "high_priority");
    assert_eq!(sorted_ops[1].id, "medium_priority");
    assert_eq!(sorted_ops[2].id, "low_priority");

    assert_eq!(sorted_ops[0].priority, 1);
    assert_eq!(sorted_ops[1].priority, 2);
    assert_eq!(sorted_ops[2].priority, 3);
}

#[tokio::test]
async fn test_cache_hit_ratio_calculation() {
    // Simulate cache metrics
    struct CacheStats {
        hits: u64,
        misses: u64,
        total_requests: u64,
    }

    let stats = CacheStats {
        hits: 850,
        misses: 150,
        total_requests: 1000,
    };

    let hit_ratio = stats.hits as f64 / stats.total_requests as f64;
    assert_eq!(hit_ratio, 0.85);

    // Test edge cases
    let empty_stats = CacheStats {
        hits: 0,
        misses: 0,
        total_requests: 0,
    };

    let empty_ratio = if empty_stats.total_requests > 0 {
        empty_stats.hits as f64 / empty_stats.total_requests as f64
    } else {
        0.0
    };

    assert_eq!(empty_ratio, 0.0);
}
