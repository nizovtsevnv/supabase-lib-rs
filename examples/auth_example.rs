//! Authentication example for Supabase Rust client

use std::env;
use supabase_lib_rs::prelude::*;

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let supabase_url =
        env::var("SUPABASE_URL").expect("SUPABASE_URL environment variable is required");
    let supabase_key =
        env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY environment variable is required");

    println!("🔐 Supabase Rust Client - Authentication Example");

    let client = Client::new(&supabase_url, &supabase_key)?;
    let auth = client.auth();

    println!("✅ Client initialized");

    // Check initial authentication status
    println!(
        "Initial auth status: {}",
        if auth.is_authenticated() {
            "✅ Authenticated"
        } else {
            "❌ Not authenticated"
        }
    );

    // Example: Sign up a new user
    let test_email = "test@example.com";
    let test_password = "testpassword123";

    println!("\n📝 Attempting to sign up user: {}", test_email);
    match auth
        .sign_up_with_email_and_password(test_email, test_password)
        .await
    {
        Ok(response) => {
            if let Some(user) = response.user {
                println!("✅ Sign up successful!");
                println!("   User ID: {}", user.id);
                println!(
                    "   Email: {}",
                    user.email.unwrap_or_else(|| "No email".to_string())
                );
                println!("   Created: {}", user.created_at);
            } else {
                println!("⚠️ Sign up response received but no user data");
            }

            if let Some(session) = response.session {
                println!("   Session expires at: {}", session.expires_at);
                println!("   Token type: {}", session.token_type);
            }
        }
        Err(e) => {
            println!("❌ Sign up failed: {}", e);
            println!("   This is expected if the user already exists or if Supabase is not properly configured");
        }
    }

    // Example: Sign in with email and password
    println!("\n🔑 Attempting to sign in user: {}", test_email);
    match auth
        .sign_in_with_email_and_password(test_email, test_password)
        .await
    {
        Ok(response) => {
            if let Some(user) = response.user {
                println!("✅ Sign in successful!");
                println!("   User ID: {}", user.id);
                println!(
                    "   Email: {}",
                    user.email.unwrap_or_else(|| "No email".to_string())
                );
                println!(
                    "   Last sign in: {}",
                    user.last_sign_in_at
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "Never".to_string())
                );
            }

            if let Some(session) = response.session {
                println!(
                    "   Access token: {}...",
                    &session.access_token[..20.min(session.access_token.len())]
                );
                println!("   Expires in: {} seconds", session.expires_in);
                println!("   Expires at: {}", session.expires_at);
            }
        }
        Err(e) => {
            println!("❌ Sign in failed: {}", e);
            println!("   This is expected if the user doesn't exist or credentials are wrong");
        }
    }

    // Check authentication status after sign in attempt
    println!("\n🔍 Checking authentication status...");
    println!(
        "Is authenticated: {}",
        if auth.is_authenticated() {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "Needs refresh: {}",
        if auth.needs_refresh() {
            "⚠️ Yes"
        } else {
            "✅ No"
        }
    );

    // Get current user
    match auth.current_user().await {
        Ok(Some(user)) => {
            println!("👤 Current user:");
            println!("   ID: {}", user.id);
            println!(
                "   Email: {}",
                user.email.unwrap_or_else(|| "No email".to_string())
            );
            println!(
                "   Role: {}",
                user.role.unwrap_or_else(|| "No role".to_string())
            );
            println!("   Audience: {}", user.aud);
        }
        Ok(None) => {
            println!("👤 No current user");
        }
        Err(e) => {
            println!("❌ Failed to get current user: {}", e);
        }
    }

    // Example: Password reset
    println!("\n📧 Attempting password reset for: {}", test_email);
    match auth.reset_password_for_email(test_email).await {
        Ok(_) => {
            println!("✅ Password reset email sent successfully!");
        }
        Err(e) => {
            println!("❌ Password reset failed: {}", e);
            println!("   This is expected without proper email configuration");
        }
    }

    // Example: Update user (requires authentication)
    println!("\n👤 Attempting to update user profile...");
    let user_data = serde_json::json!({
        "display_name": "Test User",
        "bio": "A test user created by the Rust client"
    });

    match auth.update_user(None, None, Some(user_data)).await {
        Ok(response) => {
            if let Some(user) = response.user {
                println!("✅ User profile updated!");
                println!("   Metadata: {:?}", user.user_metadata);
            }
        }
        Err(e) => {
            println!("❌ User update failed: {}", e);
            println!("   This is expected without proper authentication");
        }
    }

    // Example: Refresh session (requires existing session)
    println!("\n🔄 Attempting to refresh session...");
    match auth.refresh_session().await {
        Ok(response) => {
            if let Some(session) = response.session {
                println!("✅ Session refreshed successfully!");
                println!("   New expires at: {}", session.expires_at);
            }
        }
        Err(e) => {
            println!("❌ Session refresh failed: {}", e);
            println!("   This is expected without an existing session");
        }
    }

    // Auto refresh test
    println!("\n🔄 Testing auto refresh...");
    if auth.needs_refresh() {
        match auth.refresh_session().await {
            Ok(_) => {
                println!("✅ Token refresh successful");
            }
            Err(e) => {
                println!("❌ Token refresh failed: {}", e);
            }
        }
    }

    // Example: Sign out (requires authentication)
    println!("\n👋 Attempting to sign out...");
    match auth.sign_out().await {
        Ok(_) => {
            println!("✅ Sign out successful!");
        }
        Err(e) => {
            println!("❌ Sign out failed: {}", e);
            println!("   This is expected without proper authentication");
        }
    }

    // ==== Multi-Factor Authentication (MFA) Examples ====

    println!("\n=== MFA Demonstrations ===");

    // List MFA factors
    match client.auth().list_mfa_factors().await {
        Ok(factors) => {
            println!("Current MFA factors: {}", factors.len());
            for factor in factors {
                println!(
                    "- {} ({}): {}",
                    factor.friendly_name, factor.factor_type, factor.status
                );
            }
        }
        Err(e) => println!("Failed to list MFA factors: {}", e),
    }

    // Setup TOTP (Time-based One-Time Password)
    match client.auth().setup_totp("My Authenticator App").await {
        Ok(totp_setup) => {
            println!("TOTP Setup successful!");
            println!("Secret: {}", totp_setup.secret);
            println!("QR Code:\n{}", totp_setup.qr_code);
            println!("URI: {}", totp_setup.uri);

            // Generate test code (for development only)
            if let Ok(test_code) = client.auth().generate_totp_code(&totp_setup.secret) {
                println!("Generated test code: {}", test_code);
            }
        }
        Err(e) => println!("TOTP setup failed: {}", e),
    }

    // Setup SMS MFA with international phone number
    match client
        .auth()
        .setup_sms_mfa("+1-555-123-4567", "My Phone", Some("US"))
        .await
    {
        Ok(sms_factor) => {
            println!("SMS MFA configured!");
            println!("Factor ID: {}", sms_factor.id);
            if let Some(phone) = sms_factor.phone {
                println!("Phone: {}", phone);
            }
        }
        Err(e) => println!("SMS MFA setup failed: {}", e),
    }

    // ==== Advanced OAuth Token Management ====

    println!("\n=== Advanced Token Management ===");

    // Get current token metadata
    match client.auth().get_token_metadata() {
        Ok(Some(metadata)) => {
            println!("Token Metadata:");
            println!("- Issued at: {}", metadata.issued_at);
            println!("- Expires at: {}", metadata.expires_at);
            println!("- Refresh count: {}", metadata.refresh_count);
            println!("- Scopes: {:?}", metadata.scopes);
            if let Some(device_id) = metadata.device_id {
                println!("- Device ID: {}", device_id);
            }
        }
        Ok(None) => println!("No active session for token metadata"),
        Err(e) => println!("Failed to get token metadata: {}", e),
    }

    // Check if token needs refresh with buffer
    match client.auth().needs_refresh_with_buffer(300) {
        Ok(needs_refresh) => {
            println!("Token needs refresh (5min buffer): {}", needs_refresh);

            if needs_refresh {
                match client.auth().refresh_token_advanced().await {
                    Ok(new_session) => {
                        println!("Token refreshed successfully!");
                        println!("New expiry: {}", new_session.expires_at);
                    }
                    Err(e) => {
                        println!("Token refresh failed: {}", e);

                        // Check if error is retryable
                        if e.is_retryable() {
                            println!("Error is retryable");
                            if let Some(retry_after) = e.retry_after() {
                                println!("Retry after {} seconds", retry_after);
                            }
                        } else {
                            println!("Error is not retryable - re-authentication required");
                        }
                    }
                }
            }
        }
        Err(e) => println!("Failed to check refresh status: {}", e),
    }

    // Get time until token expiry
    match client.auth().time_until_expiry() {
        Ok(Some(seconds)) => {
            println!("Token expires in {} seconds", seconds);
            if seconds < 300 {
                println!("⚠️ Token expires soon - consider refreshing!");
            }
        }
        Ok(None) => println!("No active session"),
        Err(e) => println!("Failed to get expiry time: {}", e),
    }

    // Validate token locally (without API call)
    match client.auth().validate_token_local() {
        Ok(is_valid) => {
            println!("Token is valid locally: {}", is_valid);
            if !is_valid {
                println!("⚠️ Token is invalid or expired locally");
            }
        }
        Err(e) => println!("Token validation error: {}", e),
    }

    // Final authentication status check
    println!("\n🔍 Final authentication status:");
    println!(
        "Is authenticated: {}",
        if auth.is_authenticated() {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );

    println!("\n✨ Authentication example completed!");
    println!("💡 For full functionality, ensure Supabase Auth is properly configured.");

    Ok(())
}
