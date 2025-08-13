//! Basic usage example for Supabase Rust client

use std::env;
use supabase::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Load configuration from environment
    let supabase_url =
        env::var("SUPABASE_URL").expect("SUPABASE_URL environment variable is required");
    let supabase_key =
        env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY environment variable is required");

    println!("🚀 Supabase Rust Client - Basic Usage Example");
    println!("URL: {}", supabase_url);

    // Create client
    let client = Client::new(&supabase_url, &supabase_key)?;
    println!("✅ Client created successfully");

    // Test health check
    match client.health_check().await {
        Ok(is_healthy) => {
            println!(
                "🏥 Health check: {}",
                if is_healthy {
                    "✅ Healthy"
                } else {
                    "❌ Unhealthy"
                }
            );
        }
        Err(e) => {
            println!("⚠️ Health check failed: {}", e);
        }
    }

    // Get version information
    match client.version().await {
        Ok(version) => {
            println!("ℹ️ Version info: {:?}", version);
        }
        Err(e) => {
            println!("⚠️ Failed to get version info: {}", e);
        }
    }

    // Test authentication status
    println!(
        "🔐 Authentication status: {}",
        if client.is_authenticated() {
            "Authenticated"
        } else {
            "Not authenticated"
        }
    );

    // Test current user
    match client.current_user().await {
        Ok(Some(user)) => {
            println!(
                "👤 Current user: {} ({})",
                user.email.unwrap_or_else(|| "No email".to_string()),
                user.id
            );
        }
        Ok(None) => {
            println!("👤 No current user");
        }
        Err(e) => {
            println!("⚠️ Failed to get current user: {}", e);
        }
    }

    // Demonstrate module access
    println!("\n📦 Available modules:");
    println!("  🔐 Auth: Available");
    println!("  🗄️ Database: Available");
    println!("  📁 Storage: Available");
    println!("  ⚡ Realtime: Available");

    // Test database query builder (won't execute without proper setup)
    let _query = client
        .database()
        .from("posts")
        .select("*")
        .eq("published", "true")
        .limit(5);
    println!("  🗄️ Database query builder: ✅");

    // Test storage operations (won't execute without proper setup)
    let _storage = client.storage();
    println!("  📁 Storage client: ✅");

    // Test realtime client
    let _realtime = client.realtime();
    println!("  ⚡ Realtime client: ✅");

    println!("\n✨ Basic usage example completed successfully!");
    println!("💡 To run full integration tests, ensure you have a running Supabase instance.");

    Ok(())
}
