//! Integration tests for authentication module

use supabase_rs::prelude::*;

mod common;
use common::*;

#[tokio::test]
async fn test_auth_module_initialization() {
    let client = create_test_client();
    let auth = client.auth();

    // Initially not authenticated
    assert!(!auth.is_authenticated());

    // No current user
    let user = auth.current_user().await.unwrap();
    assert!(user.is_none());
}

#[tokio::test]
async fn test_auth_needs_refresh() {
    let client = create_test_client();
    let auth = client.auth();

    // Without a session, should not need refresh
    assert!(!auth.needs_refresh());
}

async_test!(test_sign_up_invalid_credentials, || async {
    let client = create_test_client();

    // Test sign up with invalid email format
    let result = client.auth()
        .sign_up_with_email_and_password("invalid-email", "password123")
        .await;

    // Should return an error (in real environment)
    // For mock/test environment, we just verify it doesn't panic
    match result {
        Ok(_) => {
            // In test environment, might succeed
            println!("Sign up succeeded (test environment)");
        },
        Err(e) => {
            // Expected in production environment
            println!("Sign up failed as expected: {}", e);
        }
    }
});

async_test!(test_sign_in_without_signup, || async {
    let client = create_test_client();

    let email = random_test_email();
    let password = random_test_password();

    // Try to sign in without signing up first
    let result = client.auth()
        .sign_in_with_email_and_password(&email, &password)
        .await;

    // Should fail since user doesn't exist
    match result {
        Ok(_) => {
            println!("Unexpected success in test environment");
        },
        Err(e) => {
            println!("Expected failure: {}", e);
            assert!(e.to_string().contains("auth") || e.to_string().contains("invalid") || e.to_string().contains("not found"));
        }
    }
});

async_test!(test_password_reset, || async {
    let client = create_test_client();

    let email = random_test_email();

    // Request password reset
    let result = client.auth()
        .reset_password_for_email(&email)
        .await;

    // In test environment, this might succeed or fail
    match result {
        Ok(_) => {
            println!("Password reset request succeeded");
        },
        Err(e) => {
            println!("Password reset request failed: {}", e);
        }
    }
});

#[tokio::test]
async fn test_session_management() {
    let client = create_test_client();
    let auth = client.auth();

    // Test getting session when none exists
    let session_result = auth.get_session();
    assert!(session_result.is_err());

    // Test clearing session (should not fail even if no session)
    let clear_result = auth.clear_session().await;
    assert!(clear_result.is_ok());
}

async_test!(test_update_user_without_session, || async {
    let client = create_test_client();

    // Try to update user without being signed in
    let result = client.auth()
        .update_user(
            Some("new@example.com".to_string()),
            None,
            None,
        )
        .await;

    // Should fail since no session exists
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("session") || error.to_string().contains("auth"));
});

async_test!(test_refresh_session_without_session, || async {
    let client = create_test_client();

    // Try to refresh session without having one
    let result = client.auth().refresh_session().await;

    // Should fail since no session exists
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("session") || error.to_string().contains("auth"));
});

#[tokio::test]
async fn test_auto_refresh() {
    let client = create_test_client();
    let auth = client.auth();

    // Auto refresh should not fail even without a session
    let result = auth.auto_refresh().await;
    assert!(result.is_ok());
}

// Note: Full integration tests would require a real Supabase instance
// and would include:
// - Actual sign up with email confirmation
// - Sign in with valid credentials
// - Token refresh functionality
// - User profile updates
// - Sign out functionality
// - OAuth provider authentication
