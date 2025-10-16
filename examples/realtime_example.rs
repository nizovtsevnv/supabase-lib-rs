//! Realtime subscriptions example for Supabase Rust client

use supabase_lib_rs::prelude::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let supabase_url = "http://localhost:54321";
    let supabase_key = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0";

    println!("⚡ Supabase Rust Client - Realtime Example");

    // Initialize client
    let _client = Client::new(supabase_url, supabase_key)?;

    // Note: In a real application, you'd use proper error handling
    // and implement the realtime functionality
    println!("⚠️  This is a template example.");
    println!("📝 Real implementation would require WebSocket setup");
    println!("💡 Check the documentation for complete realtime features");

    // TODO: Implement actual realtime functionality when ready
    // Example skeleton for future implementation:
    /*
    let realtime = client.realtime();

    let subscription = realtime
        .channel("public:todos")
        .on_insert(|payload| {
            println!("New todo inserted: {:?}", payload);
        })
        .subscribe()
        .await?;

    // Keep connection alive
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    */

    #[cfg(not(feature = "realtime"))]
    {
        println!("❌ Realtime feature is not enabled!");
        println!("   Run with: cargo run --example realtime_example --features realtime,native");
        return Ok(());
    }

    #[cfg(feature = "realtime")]
    {
        println!("✅ Client initialized");

        let realtime = _client.realtime();

        // Example 1: Connect to realtime WebSocket
        println!("\n🔌 Example 1: Connect to realtime");
        match realtime.connect().await {
            Ok(()) => {
                println!("✅ Connected to realtime WebSocket");
            }
            Err(e) => {
                println!("⚠️ Connection failed (expected in local dev): {}", e);
            }
        }

        println!("\n📋 Realtime functionality includes:");
        println!("   ✅ WebSocket connections");
        println!("   ✅ Channel subscriptions");
        println!("   ✅ Real-time database changes");
        println!("   ✅ Table change subscriptions");
        println!("   ✅ Event type filtering (INSERT, UPDATE, DELETE)");

        println!("\n🎉 Realtime example completed!");
        println!("   In a real application, keep subscriptions active");
        return Ok(());
    }
}
