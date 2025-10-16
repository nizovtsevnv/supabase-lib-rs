//! Advanced Realtime Tests - v0.4.1 Features
//!
//! Tests for presence system, broadcast messages, advanced filters, and connection pooling

use std::{collections::HashMap, sync::Arc};
use supabase_lib_rs::realtime::{
    PresenceState, AdvancedFilter, FilterOperator, SubscriptionConfig,
    RealtimeEvent, BroadcastMessage, ConnectionPool, ConnectionPoolConfig,
    ConnectionPoolStats
};
use crate::common::*;

#[tokio::test]
async fn test_presence_state_creation() {
    let mut metadata = HashMap::new();
    metadata.insert("status".to_string(), serde_json::json!("online"));
    metadata.insert("location".to_string(), serde_json::json!("dashboard"));

    let presence = PresenceState {
        user_id: "test_user_123".to_string(),
        online_at: chrono::Utc::now().to_rfc3339(),
        metadata: Some(metadata.clone()),
    };

    assert_eq!(presence.user_id, "test_user_123");
    assert!(presence.metadata.is_some());

    let meta = presence.metadata.unwrap();
    assert_eq!(meta.get("status").unwrap(), &serde_json::json!("online"));
    assert_eq!(meta.get("location").unwrap(), &serde_json::json!("dashboard"));
}

#[tokio::test]
async fn test_broadcast_message_structure() {
    let message = BroadcastMessage {
        event: "user_joined".to_string(),
        payload: serde_json::json!({
            "user_id": "123",
            "room": "general"
        }),
        from_user_id: Some("admin".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    assert_eq!(message.event, "user_joined");
    assert!(message.payload.is_object());
    assert_eq!(message.from_user_id, Some("admin".to_string()));

    // Test serialization
    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: BroadcastMessage = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.event, "user_joined");
}

#[tokio::test]
async fn test_advanced_filters() {
    let filters = vec![
        AdvancedFilter {
            column: "status".to_string(),
            operator: FilterOperator::Equal,
            value: serde_json::json!("published"),
        },
        AdvancedFilter {
            column: "priority".to_string(),
            operator: FilterOperator::GreaterThan,
            value: serde_json::json!(5),
        },
        AdvancedFilter {
            column: "tags".to_string(),
            operator: FilterOperator::Like,
            value: serde_json::json!("%urgent%"),
        },
        AdvancedFilter {
            column: "category".to_string(),
            operator: FilterOperator::In,
            value: serde_json::json!(["news", "updates", "alerts"]),
        },
    ];

    for filter in &filters {
        assert!(!filter.column.is_empty());
        assert!(filter.value.is_string() || filter.value.is_number() || filter.value.is_array());
    }

    // Test filter operators
    let operators = vec![
        FilterOperator::Equal,
        FilterOperator::NotEqual,
        FilterOperator::GreaterThan,
        FilterOperator::GreaterThanOrEqual,
        FilterOperator::LessThan,
        FilterOperator::LessThanOrEqual,
        FilterOperator::In,
        FilterOperator::Is,
        FilterOperator::Like,
        FilterOperator::ILike,
        FilterOperator::Match,
        FilterOperator::IMatch,
    ];

    for op in operators {
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: FilterOperator = serde_json::from_str(&serialized).unwrap();

        // Check that serialization/deserialization works
        assert_eq!(
            std::mem::discriminant(&op),
            std::mem::discriminant(&deserialized)
        );
    }
}

#[tokio::test]
async fn test_subscription_config() {
    let config = SubscriptionConfig {
        table: Some("messages".to_string()),
        schema: "public".to_string(),
        event: Some(RealtimeEvent::Insert),
        filter: Some("room_id=eq.123".to_string()),
        advanced_filters: vec![
            AdvancedFilter {
                column: "active".to_string(),
                operator: FilterOperator::Equal,
                value: serde_json::json!(true),
            }
        ],
        enable_presence: true,
        enable_broadcast: true,
        presence_callback: None,
        broadcast_callback: None,
    };

    assert_eq!(config.table, Some("messages".to_string()));
    assert_eq!(config.schema, "public");
    assert!(matches!(config.event, Some(RealtimeEvent::Insert)));
    assert_eq!(config.filter, Some("room_id=eq.123".to_string()));
    assert_eq!(config.advanced_filters.len(), 1);
    assert!(config.enable_presence);
    assert!(config.enable_broadcast);

    // Test debug formatting
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("messages"));
    assert!(debug_str.contains("public"));
}

#[tokio::test]
async fn test_subscription_config_default() {
    let default_config = SubscriptionConfig::default();

    assert_eq!(default_config.table, None);
    assert_eq!(default_config.schema, "public");
    assert_eq!(default_config.event, None);
    assert_eq!(default_config.filter, None);
    assert!(default_config.advanced_filters.is_empty());
    assert!(!default_config.enable_presence);
    assert!(!default_config.enable_broadcast);
    assert!(default_config.presence_callback.is_none());
    assert!(default_config.broadcast_callback.is_none());
}

#[tokio::test]
async fn test_connection_pool_config() {
    let config = ConnectionPoolConfig {
        max_connections: 5,
        connection_timeout: 60,
        keep_alive_interval: 45,
        reconnect_delay: 2000,
        max_reconnect_attempts: 3,
    };

    assert_eq!(config.max_connections, 5);
    assert_eq!(config.connection_timeout, 60);
    assert_eq!(config.keep_alive_interval, 45);
    assert_eq!(config.reconnect_delay, 2000);
    assert_eq!(config.max_reconnect_attempts, 3);

    // Test default config
    let default_config = ConnectionPoolConfig::default();
    assert_eq!(default_config.max_connections, 10);
    assert_eq!(default_config.connection_timeout, 30);
    assert_eq!(default_config.keep_alive_interval, 30);
    assert_eq!(default_config.reconnect_delay, 1000);
    assert_eq!(default_config.max_reconnect_attempts, 5);
}

#[cfg(feature = "realtime")]
#[tokio::test]
async fn test_connection_pool_creation() {
    let config = ConnectionPoolConfig {
        max_connections: 3,
        connection_timeout: 30,
        keep_alive_interval: 20,
        reconnect_delay: 1000,
        max_reconnect_attempts: 2,
    };

    let pool = ConnectionPool::new(config.clone());

    // Test that pool was created with correct config
    let debug_str = format!("{:?}", pool);
    assert!(debug_str.contains("ConnectionPool"));

    // Test initial stats
    let stats = pool.get_stats().await;
    assert_eq!(stats.total_connections, 3);
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.available_connections, 0);
    assert_eq!(stats.max_connections, 3);
}

#[tokio::test]
async fn test_connection_pool_stats() {
    let stats = ConnectionPoolStats {
        total_connections: 10,
        active_connections: 7,
        available_connections: 3,
        max_connections: 10,
    };

    assert_eq!(stats.total_connections, 10);
    assert_eq!(stats.active_connections, 7);
    assert_eq!(stats.available_connections, 3);
    assert_eq!(stats.max_connections, 10);

    // Test that stats can be cloned and debugged
    let cloned_stats = stats.clone();
    let debug_str = format!("{:?}", cloned_stats);
    assert!(debug_str.contains("total_connections: 10"));
    assert!(debug_str.contains("active_connections: 7"));
}

#[tokio::test]
async fn test_realtime_events() {
    let events = vec![
        RealtimeEvent::Insert,
        RealtimeEvent::Update,
        RealtimeEvent::Delete,
        RealtimeEvent::All,
    ];

    for event in events {
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: RealtimeEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            std::mem::discriminant(&event),
            std::mem::discriminant(&deserialized)
        );
    }
}

#[tokio::test]
async fn test_callback_types() {
    // Test presence callback creation
    let presence_counter = Arc::new(std::sync::Mutex::new(0));
    let counter_clone = Arc::clone(&presence_counter);

    let _presence_callback = Arc::new(move |_event| {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
    });

    // Test broadcast callback creation
    let broadcast_counter = Arc::new(std::sync::Mutex::new(0));
    let counter_clone = Arc::clone(&broadcast_counter);

    let _broadcast_callback = Arc::new(move |_message| {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
    });

    // Verify counters are properly set up
    assert_eq!(*presence_counter.lock().unwrap(), 0);
    assert_eq!(*broadcast_counter.lock().unwrap(), 0);
}

#[tokio::test]
async fn test_advanced_realtime_workflow() {
    // Test complete workflow with advanced features

    // 1. Setup presence
    let mut presence_metadata = HashMap::new();
    presence_metadata.insert("role".to_string(), serde_json::json!("user"));
    presence_metadata.insert("room".to_string(), serde_json::json!("general"));

    let presence = PresenceState {
        user_id: "workflow_user".to_string(),
        online_at: chrono::Utc::now().to_rfc3339(),
        metadata: Some(presence_metadata),
    };

    // 2. Setup broadcast message
    let broadcast = BroadcastMessage {
        event: "workflow_start".to_string(),
        payload: serde_json::json!({
            "workflow_id": "test_workflow_123",
            "status": "starting"
        }),
        from_user_id: Some("system".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // 3. Setup advanced filters
    let filters = vec![
        AdvancedFilter {
            column: "workflow_id".to_string(),
            operator: FilterOperator::Equal,
            value: serde_json::json!("test_workflow_123"),
        },
        AdvancedFilter {
            column: "priority".to_string(),
            operator: FilterOperator::GreaterThanOrEqual,
            value: serde_json::json!(1),
        },
    ];

    // 4. Setup subscription config
    let config = SubscriptionConfig {
        table: Some("workflow_events".to_string()),
        schema: "public".to_string(),
        event: Some(RealtimeEvent::All),
        filter: None,
        advanced_filters: filters,
        enable_presence: true,
        enable_broadcast: true,
        presence_callback: None,
        broadcast_callback: None,
    };

    // 5. Setup connection pool
    let pool_config = ConnectionPoolConfig::default();
    let _pool = ConnectionPool::new(pool_config);

    // Verify all components
    assert_eq!(presence.user_id, "workflow_user");
    assert_eq!(broadcast.event, "workflow_start");
    assert_eq!(config.advanced_filters.len(), 2);
    assert!(config.enable_presence);
    assert!(config.enable_broadcast);

    println!("✅ Advanced realtime workflow test completed successfully");
}
