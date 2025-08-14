//! Authentication example for Supabase Rust client

use std::env;
use supabase::prelude::*;

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
