//! Common test utilities and setup

use supabase_lib_rs::{Client, types::*};
use std::env;

/// Test configuration for integration tests
pub struct TestConfig {
    pub url: String,
    pub key: String,
}

impl TestConfig {
    /// Create test configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            url: env::var("SUPABASE_URL")
                .unwrap_or_else(|_| "http://localhost:54321".to_string()),
            key: env::var("SUPABASE_ANON_KEY")
                .unwrap_or_else(|_| "test-anon-key".to_string()),
        }
    }
}

/// Create a test client
pub fn create_test_client() -> Client {
    let config = TestConfig::from_env();
    Client::new(&config.url, &config.key).expect("Failed to create test client")
}

/// Create a test client with custom configuration
pub fn create_test_client_with_config(test_config: TestConfig) -> Client {
    let config = SupabaseConfig {
        url: test_config.url,
        key: test_config.key,
        service_role_key: None,
        http_config: HttpConfig::default(),
        auth_config: AuthConfig::default(),
        database_config: DatabaseConfig::default(),
        storage_config: StorageConfig::default(),
    };

    Client::new_with_config(config).expect("Failed to create test client")
}

/// Generate a random test email
pub fn random_test_email() -> String {
    use uuid::Uuid;
    format!("test-{}@example.com", Uuid::new_v4())
}

/// Generate a random test password
pub fn random_test_password() -> String {
    use uuid::Uuid;
    format!("password-{}", Uuid::new_v4())
}

/// Generate a random bucket name for testing
pub fn random_bucket_name() -> String {
    use uuid::Uuid;
    format!("test-bucket-{}", Uuid::new_v4())
}

/// Generate a random file name for testing
pub fn random_file_name() -> String {
    use uuid::Uuid;
    format!("test-file-{}.txt", Uuid::new_v4())
}

/// Skip test if Supabase is not available
pub fn skip_if_no_supabase() -> bool {
    env::var("SUPABASE_URL").is_err() || env::var("SUPABASE_ANON_KEY").is_err()
}

/// Test data structure for database tests
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestRecord {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TestRecord {
    pub fn new(name: &str) -> Self {
        Self {
            id: None,
            name: name.to_string(),
            description: None,
            created_at: None,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
}

/// Async test helper macro
#[macro_export]
macro_rules! async_test {
    ($test_name:ident, $test_fn:expr) => {
        #[tokio::test]
        async fn $test_name() {
            if crate::common::skip_if_no_supabase() {
                println!("Skipping test - Supabase not configured");
                return;
            }
            $test_fn().await;
        }
    };
}

/// Result type for tests
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Assert that a result is an error with specific message
#[macro_export]
macro_rules! assert_error_contains {
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(_) => panic!("Expected error but got Ok"),
            Err(e) => assert!(e.to_string().contains($msg), "Error '{}' does not contain '{}'", e, $msg),
        }
    };
}
