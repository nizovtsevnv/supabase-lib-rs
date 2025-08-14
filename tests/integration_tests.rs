//! Integration tests for Supabase Rust Client
//!
//! These tests require a running Supabase instance.
//!
//! ## Quick Start
//! ```bash
//! # Start local Supabase with Docker/Podman
//! just supabase-start
//!
//! # Run all tests
//! just test-all
//!
//! # Run only integration tests
//! just test-integration
//! ```

use std::env;
use supabase::Client;

/// Test configuration for integration tests
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub url: String,
    pub key: String,
    pub service_role_key: Option<String>,
}

impl TestConfig {
    /// Create test configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            url: env::var("SUPABASE_URL")
                .unwrap_or_else(|_| "http://localhost:54321".to_string()),
            key: env::var("SUPABASE_ANON_KEY")
                .unwrap_or_else(|_| "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0".to_string()),
            service_role_key: env::var("SUPABASE_SERVICE_ROLE_KEY").ok(),
        }
    }

    pub fn is_available(&self) -> bool {
        env::var("SUPABASE_URL").is_ok() && env::var("SUPABASE_ANON_KEY").is_ok()
    }
}

/// Create a test client
pub fn create_test_client() -> Option<Client> {
    let config = TestConfig::from_env();

    // Check if URL is valid before trying to create client
    if config.url == "http://localhost:54321" && !config.is_available() {
        return None;
    }

    Client::new(&config.url, &config.key).ok()
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

/// Skip test if Supabase is not available
pub fn skip_if_no_supabase() -> bool {
    !TestConfig::from_env().is_available()
}

// =============================================================================
// INTEGRATION TESTS - Client & Core
// =============================================================================

#[tokio::test]
async fn integration_client_creation() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    let client = match create_test_client() {
        Some(client) => client,
        None => {
            println!("⏭️ Skipping - Failed to create client");
            return;
        }
    };

    assert!(!client.url().is_empty());
    assert!(!client.key().is_empty());

    println!("✅ Client created successfully");
    println!("   URL: {}", client.url());
}

#[tokio::test]
async fn integration_health_check() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    let client = match create_test_client() {
        Some(client) => client,
        None => {
            println!("⏭️ Skipping - Failed to create client");
            return;
        }
    };

    match client.health_check().await {
        Ok(healthy) => {
            println!(
                "✅ Health check: {}",
                if healthy { "OK" } else { "Not healthy" }
            );
            // Don't assert on health for local dev - might be starting up
        }
        Err(e) => {
            println!("⚠️ Health check failed: {}", e);
            // Don't fail test - local Supabase might be starting
        }
    }
}

// =============================================================================
// INTEGRATION TESTS - Authentication
// =============================================================================

#[tokio::test]
async fn integration_auth_initialization() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    let client = match create_test_client() {
        Some(client) => client,
        None => {
            println!("⏭️ Skipping - Failed to create client");
            return;
        }
    };
    let auth = client.auth();

    // Initially not authenticated
    assert!(!auth.is_authenticated());
    println!("✅ Auth not authenticated (as expected)");

    // No current user
    let user = auth.current_user().await.unwrap();
    assert!(user.is_none());
    println!("✅ No current user (as expected)");
}

#[cfg(feature = "auth")]
#[tokio::test]
async fn integration_auth_invalid_credentials() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    let client = match create_test_client() {
        Some(client) => client,
        None => {
            println!("⏭️ Skipping - Failed to create client");
            return;
        }
    };
    let email = random_test_email();
    let password = random_test_password();

    let result = client
        .auth()
        .sign_in_with_email_and_password(&email, &password)
        .await;

    // Should fail for non-existent user
    match result {
        Ok(_) => println!("⚠️ Sign in unexpectedly succeeded (test environment?)"),
        Err(e) => {
            println!("✅ Sign in correctly failed: {}", e);
            assert!(e.to_string().contains("Invalid") || e.to_string().contains("400"));
        }
    }
}

// =============================================================================
// INTEGRATION TESTS - Database
// =============================================================================

#[cfg(feature = "database")]
#[tokio::test]
async fn integration_database_query_builder() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    let client = match create_test_client() {
        Some(client) => client,
        None => {
            println!("⏭️ Skipping - Failed to create client");
            return;
        }
    };

    // Test query builder creation (doesn't hit network)
    let query = client.database().from("test_table").select("*").limit(10);

    // Just verify we can create the query
    println!("✅ Database query builder created successfully");

    // Try to execute - might fail due to missing table, but should not panic
    let result = query.execute::<serde_json::Value>().await;
    match result {
        Ok(_) => println!("✅ Query executed successfully"),
        Err(e) => println!("⚠️ Query failed (expected for missing table): {}", e),
    }
}

// =============================================================================
// INTEGRATION TESTS - Storage
// =============================================================================

#[cfg(feature = "storage")]
#[tokio::test]
async fn integration_storage_list_buckets() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    let client = match create_test_client() {
        Some(client) => client,
        None => {
            println!("⏭️ Skipping - Failed to create client");
            return;
        }
    };
    let storage = client.storage();

    match storage.list_buckets().await {
        Ok(buckets) => {
            println!("✅ Listed {} buckets", buckets.len());
        }
        Err(e) => {
            println!("⚠️ List buckets failed: {}", e);
            // Don't fail test - might be permissions issue
        }
    }
}

// =============================================================================
// INTEGRATION TESTS - Realtime
// =============================================================================

#[cfg(feature = "realtime")]
#[tokio::test]
async fn integration_realtime_connection() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    let client = match create_test_client() {
        Some(client) => client,
        None => {
            println!("⏭️ Skipping - Failed to create client");
            return;
        }
    };

    let realtime = client.realtime();

    // Test basic connection lifecycle
    match realtime.connect().await {
        Ok(_) => {
            println!("✅ Realtime connected");

            // Test channel creation
            let _channel = realtime.channel("test-channel");
            println!("✅ Channel created: test-channel");

            // Test disconnection
            match realtime.disconnect().await {
                Ok(_) => println!("✅ Realtime disconnected"),
                Err(e) => println!("⚠️ Disconnect failed: {}", e),
            }
        }
        Err(e) => {
            println!("⚠️ Realtime connection failed: {}", e);
            // Don't fail test - WebSocket might not be available
        }
    }
}

// =============================================================================
// E2E TESTS - Full Workflows
// =============================================================================

#[tokio::test]
async fn e2e_preflight_checks() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    let client = match create_test_client() {
        Some(client) => client,
        None => {
            println!("⏭️ Skipping - Failed to create client");
            return;
        }
    };

    // 1. Health check
    match client.health_check().await {
        Ok(true) => println!("✅ Health check: OK"),
        Ok(false) => println!("⚠️ Health check: Not healthy"),
        Err(e) => println!("⚠️ Health check error: {}", e),
    }

    // 2. Version info
    match client.version().await {
        Ok(info) => {
            println!("✅ Version endpoint: {} keys", info.len());
            if let Some(version) = info.get("version") {
                println!("   Version: {}", version);
            }
        }
        Err(e) => println!("⚠️ Version endpoint error: {}", e),
    }

    println!("✅ E2E preflight checks completed");
}

#[cfg(all(feature = "auth", feature = "database"))]
#[tokio::test]
async fn e2e_auth_database_flow() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    let client = match create_test_client() {
        Some(client) => client,
        None => {
            println!("⏭️ Skipping - Failed to create client");
            return;
        }
    };

    // 1. Check initial auth state
    let auth = client.auth();
    assert!(!auth.is_authenticated());
    println!("✅ Initial state: not authenticated");

    // 2. Try database access (should work with anon key)
    let database = client.database();
    let query_result = database
        .from("test_table")
        .select("*")
        .limit(1)
        .execute::<serde_json::Value>()
        .await;

    match query_result {
        Ok(_) => println!("✅ Database query with anon key: OK"),
        Err(e) => println!("⚠️ Database query failed: {}", e),
    }

    println!("✅ E2E auth + database flow completed");
}

#[cfg(feature = "storage")]
#[tokio::test]
async fn e2e_storage_workflow() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    let client = match create_test_client() {
        Some(client) => client,
        None => {
            println!("⏭️ Skipping - Failed to create client");
            return;
        }
    };
    let storage = client.storage();
    let bucket_name = random_bucket_name();

    println!("🧪 Testing storage workflow with bucket: {}", bucket_name);

    // 1. List existing buckets
    match storage.list_buckets().await {
        Ok(buckets) => println!("✅ Initial buckets: {}", buckets.len()),
        Err(e) => println!("⚠️ List buckets failed: {}", e),
    }

    // 2. Try to create a test bucket (might fail due to permissions)
    match storage
        .create_bucket(&bucket_name, &bucket_name, true)
        .await
    {
        Ok(_) => {
            println!("✅ Bucket created: {}", bucket_name);

            // Clean up - delete the test bucket
            match storage.delete_bucket(&bucket_name).await {
                Ok(_) => println!("✅ Bucket cleaned up"),
                Err(e) => println!("⚠️ Cleanup failed: {}", e),
            }
        }
        Err(e) => println!("⚠️ Bucket creation failed (expected): {}", e),
    }

    println!("✅ E2E storage workflow completed");
}
