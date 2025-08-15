//! Advanced Storage & Realtime Example - v0.4.1 Features
//!
//! This example demonstrates the new features introduced in v0.4.1:
//! - Resumable uploads for large files
//! - Advanced metadata (tags, custom metadata, search)
//! - Presence system for user tracking
//! - Broadcast messages for cross-client communication
//! - Advanced filters for realtime subscriptions

use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use supabase::{
    storage::{FileMetadata, ResumableUploadConfig, SearchOptions},
    Client,
};

#[cfg(feature = "realtime")]
use supabase::realtime::{
    AdvancedFilter, FilterOperator, PresenceState, RealtimeEvent, SubscriptionConfig,
};

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Storage & Realtime Demo (v0.4.1)");
    println!("==============================================\n");

    let client = Client::new("https://your-project.supabase.co", "your-anon-key")?;

    // ==========================
    // 📁 ADVANCED STORAGE DEMO
    // ==========================

    println!("📁 Advanced Storage Features Demo");
    println!("----------------------------------");

    let _storage = client.storage();

    // 1. File Metadata Management
    println!("\n🏷️  File Metadata Example:");

    let mut tags = HashMap::new();
    tags.insert("category".to_string(), "documents".to_string());
    tags.insert("project".to_string(), "web-app".to_string());
    tags.insert("priority".to_string(), "high".to_string());

    let mut custom_metadata = HashMap::new();
    custom_metadata.insert("author".to_string(), json!("john_doe"));
    custom_metadata.insert("version".to_string(), json!(1));
    custom_metadata.insert("created_by".to_string(), json!("admin_tool"));

    let file_metadata = FileMetadata {
        tags: Some(tags),
        custom_metadata: Some(custom_metadata),
        description: Some("Project documentation file".to_string()),
        category: Some("documentation".to_string()),
        searchable_content: Some("project guide documentation tutorial setup".to_string()),
    };

    // In real usage, you'd first upload a file, then update its metadata
    println!("   📋 File metadata structure: {:?}", file_metadata);
    println!("   ✨ This metadata would be attached to files for rich organization");

    // 2. File Search with Advanced Metadata
    println!("\n🔍 Advanced File Search Example:");

    let mut search_tags = HashMap::new();
    search_tags.insert("category".to_string(), "documents".to_string());

    let search_options = SearchOptions {
        tags: Some(search_tags),
        category: Some("documentation".to_string()),
        content_search: Some("project guide".to_string()),
        limit: Some(20),
        offset: Some(0),
    };

    println!("   🔎 Search criteria: {:?}", search_options);
    println!("   📝 This would find files matching tags, category, and content");

    // 3. Resumable Upload Configuration
    println!("\n📤 Resumable Upload Configuration:");

    let upload_config = ResumableUploadConfig {
        chunk_size: 5 * 1024 * 1024, // 5MB chunks
        max_retries: 3,
        retry_delay: 1000, // 1 second
        verify_checksums: true,
    };

    let _progress_callback = Arc::new(|uploaded: u64, total: u64| {
        let percent = (uploaded as f64 / total as f64) * 100.0;
        println!(
            "     📊 Upload progress: {:.1}% ({}/{})",
            percent, uploaded, total
        );
    });

    println!("   ⚙️  Upload config: {:?}", upload_config);
    println!("   📈 Progress callback configured for real-time updates");
    println!("   💡 Large files would be uploaded in chunks with resume capability");

    // ==========================
    // 📡 ADVANCED REALTIME DEMO
    // ==========================

    println!("\n📡 Advanced Realtime Features Demo");
    println!("-----------------------------------");

    #[cfg(feature = "realtime")]
    {
        let realtime = client.realtime();

        // Connect to realtime
        println!("\n🔗 Connecting to realtime server...");
        realtime.connect().await?;
        println!("   ✅ Connected successfully!");

        // 1. Presence System Demo
        println!("\n👥 Presence System Example:");

        let mut presence_metadata = HashMap::new();
        presence_metadata.insert("status".to_string(), json!("online"));
        presence_metadata.insert("location".to_string(), json!("dashboard"));
        presence_metadata.insert("device".to_string(), json!("desktop"));

        let presence_state = PresenceState {
            user_id: "user123".to_string(),
            online_at: chrono::Utc::now().to_rfc3339(),
            metadata: Some(presence_metadata),
        };

        println!("   👤 User presence: {:?}", presence_state);
        println!("   💡 This would track user as online in 'lobby' channel");

        // 2. Broadcast Messages Demo
        println!("\n📢 Broadcast Messages Example:");

        let broadcast_payload = json!({
            "message": "Welcome to the app!",
            "from": "system",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": {
                "type": "announcement",
                "priority": "high"
            }
        });

        println!("   📨 Broadcast message: {}", broadcast_payload);
        println!("   💡 This message would be sent to all users in 'announcements' channel");

        // 3. Advanced Filters Demo
        println!("\n🎯 Advanced Filters Example:");

        let advanced_filters = vec![
            AdvancedFilter {
                column: "status".to_string(),
                operator: FilterOperator::Equal,
                value: json!("published"),
            },
            AdvancedFilter {
                column: "priority".to_string(),
                operator: FilterOperator::GreaterThan,
                value: json!(3),
            },
            AdvancedFilter {
                column: "tags".to_string(),
                operator: FilterOperator::Like,
                value: json!("%urgent%"),
            },
        ];

        println!("   🔧 Advanced filters: {:?}", advanced_filters);

        // 4. Advanced Subscription Configuration
        println!("\n🔔 Advanced Subscription Example:");

        let subscription_config = SubscriptionConfig {
            table: Some("posts".to_string()),
            schema: "public".to_string(),
            event: Some(RealtimeEvent::All),
            filter: Some("user_id=eq.123".to_string()),
            advanced_filters: advanced_filters.clone(),
            enable_presence: true,
            enable_broadcast: true,
            presence_callback: Some(Arc::new(|event| {
                println!("     👥 Presence event: {:?}", event);
            })),
            broadcast_callback: Some(Arc::new(|message| {
                println!("     📢 Broadcast message: {:?}", message);
            })),
        };

        println!(
            "   ⚙️  Subscription config: table={:?}, filters={}",
            subscription_config.table,
            subscription_config.advanced_filters.len()
        );
        println!(
            "   ✅ Presence tracking: {}",
            subscription_config.enable_presence
        );
        println!(
            "   ✅ Broadcast messages: {}",
            subscription_config.enable_broadcast
        );

        println!("\n💡 Advanced subscription would provide:");
        println!("   • Real-time database changes with complex filters");
        println!("   • User presence tracking (who's online/offline)");
        println!("   • Cross-client broadcast messaging");
        println!("   • Multiple event callbacks for different message types");

        // Disconnect
        println!("\n🔌 Disconnecting from realtime server...");
        realtime.disconnect().await?;
        println!("   ✅ Disconnected successfully!");
    }

    #[cfg(not(feature = "realtime"))]
    {
        println!("   ❌ Realtime features require 'realtime' feature flag");
        println!(
            "   💡 Run with: cargo run --example storage_advanced_example --features realtime"
        );
    }

    // ==========================
    // 🎯 INTEGRATION EXAMPLE
    // ==========================

    println!("\n🎯 Integration Example: File Upload with Notifications");
    println!("-----------------------------------------------------");

    println!("💡 Real-world workflow:");
    println!("   1. Upload large file with resumable upload");
    println!("   2. Add rich metadata (tags, custom data)");
    println!("   3. Notify users via broadcast message");
    println!("   4. Update user presence (show as 'uploading')");
    println!("   5. Use advanced filters to show relevant files only");

    println!("\n✨ This demonstrates how v0.4.1 features work together!");

    Ok(())
}
