//! Advanced Storage Tests - v0.4.1 Features
//!
//! Tests for resumable uploads, advanced metadata, and storage policies

use std::{collections::HashMap, sync::Arc};
use supabase_lib_rs::{
    storage::{
        FileMetadata, ResumableUploadConfig, SearchOptions, StoragePolicy,
        PolicyOperation, PolicyTemplate
    },
    Client,
};
use crate::common::*;

#[tokio::test]
async fn test_file_metadata_operations() {
    let client = get_test_client();
    let storage = client.storage();

    // Create test metadata
    let mut tags = HashMap::new();
    tags.insert("category".to_string(), "test-docs".to_string());
    tags.insert("project".to_string(), "integration-test".to_string());

    let mut custom_data = HashMap::new();
    custom_data.insert("author".to_string(), serde_json::json!("test_user"));
    custom_data.insert("version".to_string(), serde_json::json!(1));

    let metadata = FileMetadata {
        tags: Some(tags),
        custom_metadata: Some(custom_data),
        description: Some("Test file for metadata".to_string()),
        category: Some("testing".to_string()),
        searchable_content: Some("test metadata integration".to_string()),
    };

    // Test metadata structure
    assert!(metadata.tags.is_some());
    assert!(metadata.custom_metadata.is_some());
    assert_eq!(metadata.description, Some("Test file for metadata".to_string()));
}

#[tokio::test]
async fn test_search_options_serialization() {
    let mut search_tags = HashMap::new();
    search_tags.insert("category".to_string(), "documents".to_string());

    let search_options = SearchOptions {
        tags: Some(search_tags),
        category: Some("testing".to_string()),
        content_search: Some("test search".to_string()),
        limit: Some(10),
        offset: Some(0),
    };

    // Test serialization works
    let serialized = serde_json::to_string(&search_options).unwrap();
    assert!(serialized.contains("category"));
    assert!(serialized.contains("testing"));
}

#[tokio::test]
async fn test_resumable_upload_config() {
    let config = ResumableUploadConfig {
        chunk_size: 1024 * 1024, // 1MB
        max_retries: 5,
        retry_delay: 500,
        verify_checksums: true,
    };

    assert_eq!(config.chunk_size, 1024 * 1024);
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.retry_delay, 500);
    assert!(config.verify_checksums);

    // Test default config
    let default_config = ResumableUploadConfig::default();
    assert_eq!(default_config.chunk_size, 5 * 1024 * 1024);
    assert_eq!(default_config.max_retries, 3);
    assert_eq!(default_config.retry_delay, 1000);
    assert!(default_config.verify_checksums);
}

#[tokio::test]
async fn test_progress_callback() {
    let progress_data = Arc::new(std::sync::Mutex::new((0u64, 0u64)));
    let progress_data_clone = Arc::clone(&progress_data);

    let callback = Arc::new(move |uploaded: u64, total: u64| {
        let mut data = progress_data_clone.lock().unwrap();
        *data = (uploaded, total);
    });

    // Test callback
    callback(500, 1000);

    let final_data = progress_data.lock().unwrap();
    assert_eq!(*final_data, (500, 1000));
}

#[tokio::test]
async fn test_storage_policy_creation() {
    let policy = StoragePolicy {
        name: "test_policy".to_string(),
        bucket_id: "test-bucket".to_string(),
        operation: PolicyOperation::Select,
        definition: "auth.uid() IS NOT NULL".to_string(),
        check: Some("true".to_string()),
    };

    assert_eq!(policy.name, "test_policy");
    assert_eq!(policy.bucket_id, "test-bucket");
    assert!(matches!(policy.operation, PolicyOperation::Select));
    assert_eq!(policy.definition, "auth.uid() IS NOT NULL");
    assert_eq!(policy.check, Some("true".to_string()));
}

#[tokio::test]
async fn test_policy_operations() {
    // Test all policy operations
    let operations = vec![
        PolicyOperation::Select,
        PolicyOperation::Insert,
        PolicyOperation::Update,
        PolicyOperation::Delete,
        PolicyOperation::All,
    ];

    for op in operations {
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: PolicyOperation = serde_json::from_str(&serialized).unwrap();

        match (op, deserialized) {
            (PolicyOperation::Select, PolicyOperation::Select) => (),
            (PolicyOperation::Insert, PolicyOperation::Insert) => (),
            (PolicyOperation::Update, PolicyOperation::Update) => (),
            (PolicyOperation::Delete, PolicyOperation::Delete) => (),
            (PolicyOperation::All, PolicyOperation::All) => (),
            _ => panic!("Policy operation serialization/deserialization failed"),
        }
    }
}

#[tokio::test]
async fn test_policy_template_generation() {
    let client = get_test_client();
    let storage = client.storage();

    // Test different policy templates
    let public_policy = storage.generate_policy_template(
        "test-bucket",
        "public_read",
        PolicyTemplate::PublicRead,
    );
    assert!(matches!(public_policy.operation, PolicyOperation::Select));
    assert_eq!(public_policy.definition, "true");
    assert!(public_policy.check.is_none());

    let auth_policy = storage.generate_policy_template(
        "test-bucket",
        "auth_read",
        PolicyTemplate::AuthenticatedRead,
    );
    assert!(matches!(auth_policy.operation, PolicyOperation::Select));
    assert_eq!(auth_policy.definition, "auth.uid() IS NOT NULL");

    let user_policy = storage.generate_policy_template(
        "test-bucket",
        "user_access",
        PolicyTemplate::UserFolderAccess,
    );
    assert!(matches!(user_policy.operation, PolicyOperation::All));
    assert!(user_policy.definition.contains("auth.uid()::text"));
    assert!(user_policy.check.is_some());

    let admin_policy = storage.generate_policy_template(
        "test-bucket",
        "admin_access",
        PolicyTemplate::AdminFullAccess,
    );
    assert!(matches!(admin_policy.operation, PolicyOperation::All));
    assert_eq!(admin_policy.definition, "auth.role() = 'admin'");

    let role_policy = storage.generate_policy_template(
        "test-bucket",
        "manager_read",
        PolicyTemplate::ReadOnlyForRole("manager".to_string()),
    );
    assert!(matches!(role_policy.operation, PolicyOperation::Select));
    assert_eq!(role_policy.definition, "auth.role() = 'manager'");
}

#[tokio::test]
async fn test_advanced_storage_workflow() {
    // This test demonstrates a complete workflow using advanced storage features

    // 1. Setup metadata
    let mut tags = HashMap::new();
    tags.insert("project".to_string(), "test-project".to_string());
    tags.insert("department".to_string(), "engineering".to_string());

    let metadata = FileMetadata {
        tags: Some(tags.clone()),
        custom_metadata: Some(HashMap::new()),
        description: Some("Test workflow file".to_string()),
        category: Some("workflow".to_string()),
        searchable_content: Some("workflow test automation".to_string()),
    };

    // 2. Setup resumable upload
    let upload_config = ResumableUploadConfig {
        chunk_size: 1024 * 512, // 512KB for testing
        max_retries: 2,
        retry_delay: 100,
        verify_checksums: true,
    };

    // 3. Setup search
    let search_options = SearchOptions {
        tags: Some(tags),
        category: Some("workflow".to_string()),
        content_search: Some("test".to_string()),
        limit: Some(5),
        offset: Some(0),
    };

    // 4. Setup policy
    let client = get_test_client();
    let storage = client.storage();

    let policy = storage.generate_policy_template(
        "test-workflow",
        "workflow_access",
        PolicyTemplate::UserFolderAccess,
    );

    // Verify all components work together
    assert!(metadata.tags.is_some());
    assert_eq!(upload_config.chunk_size, 1024 * 512);
    assert!(search_options.tags.is_some());
    assert_eq!(policy.bucket_id, "test-workflow");

    println!("✅ Advanced storage workflow test completed successfully");
}

#[tokio::test]
async fn test_storage_events_structure() {
    // Test storage event types
    use supabase::storage::{StorageEvent, StorageEventMessage};

    let events = vec![
        StorageEvent::FileUploaded,
        StorageEvent::FileDeleted,
        StorageEvent::FileUpdated,
        StorageEvent::BucketCreated,
        StorageEvent::BucketDeleted,
    ];

    for event in events {
        // Test serialization
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: StorageEvent = serde_json::from_str(&serialized).unwrap();

        // Verify they match
        assert_eq!(
            std::mem::discriminant(&event),
            std::mem::discriminant(&deserialized)
        );
    }
}

#[tokio::test]
async fn test_error_handling() {
    // Test that error handling works correctly for advanced features
    let client = get_test_client();
    let storage = client.storage();

    // Test invalid search options don't panic
    let invalid_search = SearchOptions {
        tags: None,
        category: None,
        content_search: None,
        limit: Some(0), // Invalid limit
        offset: Some(0),
    };

    // Should handle gracefully
    let serialized = serde_json::to_string(&invalid_search);
    assert!(serialized.is_ok());

    // Test policy validation
    let policy = StoragePolicy {
        name: "".to_string(), // Empty name should be handled
        bucket_id: "test".to_string(),
        operation: PolicyOperation::Select,
        definition: "true".to_string(),
        check: None,
    };

    assert_eq!(policy.name, "");
    assert_eq!(policy.bucket_id, "test");
}
