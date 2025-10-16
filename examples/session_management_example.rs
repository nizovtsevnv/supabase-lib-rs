//! Session Management Example
//!
//! This example demonstrates the advanced session management capabilities
//! including cross-tab synchronization, session persistence, and monitoring.

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // This example requires the session-management feature to be enabled
    #[cfg(feature = "session-management")]
    {
        use chrono::Utc;

        use supabase_lib_rs::prelude::*;
        use supabase_lib_rs::session::{
            storage::create_default_storage, SessionEvent, SessionManager, SessionManagerConfig,
        };
        use uuid::Uuid;

        println!("🚀 Advanced Session Management Demo");
        println!("====================================\n");

        // Initialize the Supabase client
        let _client = Client::new("https://your-project.supabase.co", "your-anon-key")?;

        println!("✨ Creating Session Manager...");

        // Create session manager with default storage
        let storage_backend = create_default_storage()?;
        let config = SessionManagerConfig {
            storage_backend,
            enable_cross_tab_sync: true,
            session_key_prefix: "demo_session_".to_string(),
            default_expiry_seconds: 3600, // 1 hour
            enable_encryption: false,
            encryption_key: None,
            enable_monitoring: true,
            max_memory_sessions: 100,
            sync_interval_seconds: 30,
        };

        let session_manager = SessionManager::new(config);

        // Initialize the session manager
        session_manager.initialize().await?;
        println!("✅ Session Manager initialized");

        // Setup event listener
        let event_listener_id = session_manager.on_session_event(|event| match event {
            SessionEvent::Created { session_id } => {
                println!("🎉 New session created: {}", session_id);
            }
            SessionEvent::Updated {
                session_id,
                changes,
            } => {
                println!("🔄 Session {} updated: {:?}", session_id, changes);
            }
            SessionEvent::Accessed {
                session_id,
                timestamp,
            } => {
                println!("👁️  Session {} accessed at {}", session_id, timestamp);
            }
            SessionEvent::Destroyed { session_id, reason } => {
                println!("💥 Session {} destroyed: {}", session_id, reason);
            }
            _ => {
                println!("📡 Session event: {:?}", event);
            }
        });

        println!("📡 Event listener setup complete");

        // Create a mock session for demonstration
        println!("\n🔐 Creating Demo Session...");
        let demo_user = supabase_lib_rs::auth::User {
            id: Uuid::new_v4(),
            email: Some("demo@example.com".to_string()),
            phone: None,
            email_confirmed_at: Some(Utc::now()),
            phone_confirmed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_sign_in_at: Some(Utc::now()),
            app_metadata: serde_json::json!({"provider": "email", "providers": ["email"]}),
            user_metadata: serde_json::json!({"full_name": "Demo User"}),
            aud: "authenticated".to_string(),
            role: Some("authenticated".to_string()),
        };

        let demo_session = supabase_lib_rs::auth::Session {
            access_token: "demo_access_token_12345".to_string(),
            refresh_token: "demo_refresh_token_67890".to_string(),
            expires_in: 3600,
            expires_at: Utc::now() + chrono::Duration::seconds(3600),
            token_type: "bearer".to_string(),
            user: demo_user,
        };

        // Store the session
        let session_id = session_manager.store_session(demo_session.clone()).await?;
        println!("✅ Session stored with ID: {}", session_id);

        // Demonstrate session retrieval
        println!("\n📥 Retrieving Session...");
        if let Some(session_data) = session_manager.get_session(session_id).await? {
            println!("✅ Session retrieved successfully!");
            println!(
                "   User: {}",
                session_data
                    .session
                    .user
                    .email
                    .unwrap_or_else(|| "No email".to_string())
            );
            println!("   Source: {:?}", session_data.metadata.source);
            println!("   Created: {}", session_data.metadata.created_at);
            println!(
                "   Last Accessed: {}",
                session_data.metadata.last_accessed_at
            );

            // Create updated session for demonstration
            let mut updated_session = demo_session.clone();
            updated_session.access_token = "updated_access_token_98765".to_string();

            // Update the session
            println!("\n🔄 Updating Session...");
            session_manager
                .update_session(session_id, updated_session)
                .await?;
            println!("✅ Session updated successfully!");
        }

        // List all active sessions
        println!("\n📋 Listing All Sessions...");
        let sessions = session_manager.list_sessions().await?;
        println!("📊 Found {} active session(s)", sessions.len());

        for (index, session_data) in sessions.iter().enumerate() {
            println!(
                "   {}. Session ID: {}",
                index + 1,
                session_data.metadata.session_id
            );
            println!(
                "      User: {}",
                session_data
                    .session
                    .user
                    .email
                    .as_deref()
                    .unwrap_or("No email")
            );
            println!("      Expires: {}", session_data.session.expires_at);
        }

        // Demonstrate session monitoring
        println!("\n👁️  Session Monitoring Demo...");

        // Simulate accessing the session multiple times
        for i in 1..=3 {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            session_manager.get_session(session_id).await?;
            println!("   Access #{} completed", i);
        }

        // Clean up - remove the session
        println!("\n🧹 Cleanup - Removing Session...");
        session_manager
            .remove_session(session_id, "Demo completed".to_string())
            .await?;
        println!("✅ Session removed successfully");

        // Remove event listener
        session_manager.remove_event_listener(event_listener_id);
        println!("✅ Event listener removed");

        println!("\n🎯 Session Management Demo Completed!");
        println!("=====================================");

        Ok(())
    }

    #[cfg(not(feature = "session-management"))]
    {
        println!("❌ This example requires the 'session-management' feature to be enabled.");
        println!("   Run with: cargo run --example session_management_example --features session-management");
        Ok(())
    }
}
