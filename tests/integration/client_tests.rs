//! Integration tests for the main client

use supabase_rs::Client;

mod common;
use common::*;

#[tokio::test]
async fn test_client_creation() {
    let result = Client::new("https://example.supabase.co", "test-key");
    assert!(result.is_ok());

    let client = result.unwrap();
    assert_eq!(client.url(), "https://example.supabase.co");
    assert_eq!(client.key(), "test-key");
}

#[tokio::test]
async fn test_client_invalid_url() {
    let result = Client::new("invalid-url", "test-key");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_client_modules_available() {
    let client = create_test_client();

    // Test that all modules are accessible
    let _auth = client.auth();
    let _database = client.database();
    let _storage = client.storage();
    let _realtime = client.realtime();
}

#[tokio::test]
async fn test_client_authentication_state() {
    let client = create_test_client();

    // Initially should not be authenticated
    assert!(!client.is_authenticated());

    // Current user should be None
    let user = client.current_user().await.unwrap();
    assert!(user.is_none());
}

async_test!(test_health_check_mock, || async {
    // This test would require a mock server or actual Supabase instance
    // For now, just test the client creation
    let client = create_test_client();

    // In a real test environment, you would:
    // let is_healthy = client.health_check().await.unwrap();
    // assert!(is_healthy);

    // For now, just verify client is created
    assert!(!client.url().is_empty());
});

#[tokio::test]
async fn test_client_config() {
    let config = TestConfig::from_env();
    let client = create_test_client_with_config(config);

    assert!(!client.url().is_empty());
    assert!(!client.key().is_empty());
}
