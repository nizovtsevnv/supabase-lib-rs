//! Realtime subscriptions example for Supabase Rust client

use std::env;
#[cfg(not(feature = "realtime"))]
use supabase::prelude::*;
#[cfg(feature = "realtime")]
use supabase::{prelude::*, realtime::RealtimeEvent};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let supabase_url =
        env::var("SUPABASE_URL").expect("SUPABASE_URL environment variable is required");
    let supabase_key =
        env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY environment variable is required");

    println!("⚡ Supabase Rust Client - Realtime Example");

    let client = Client::new(&supabase_url, &supabase_key)?;

    #[cfg(not(feature = "realtime"))]
    {
        println!("❌ Realtime feature is not enabled!");
        println!("   Run with: cargo run --example realtime_example --features realtime,native");
        return Ok(());
    }

    #[cfg(feature = "realtime")]
    {
        println!("✅ Client initialized");

        let realtime = client.realtime();

        // Example 1: Connect to realtime WebSocket
        println!("\n⚡ Example 1: Connect to realtime");
        match realtime.connect().await {
            Ok(_) => {
                println!("✅ Connected to realtime WebSocket successfully!");
            }
            Err(e) => {
                println!("❌ Realtime connection failed: {}", e);
                println!("   This is expected without a properly configured Supabase instance");
            }
        }

        // Example 2: Basic subscription
        println!("\n📡 Example 2: Subscribe to table changes");
        match realtime
            .channel("posts")
            .table("posts")
            .subscribe(|message| {
                println!("📨 Received message: {:?}", message.event);
            })
            .await
        {
            Ok(subscription_id) => {
                println!("✅ Subscription created with ID: {}", subscription_id);
            }
            Err(e) => {
                println!("❌ Subscription failed: {}", e);
            }
        }

        // Example 3: Filtered subscription
        println!("\n🔍 Example 3: Subscribe with event filter");
        match realtime
            .channel("posts_inserts")
            .table("posts")
            .event(RealtimeEvent::Insert)
            .subscribe(|message| {
                println!("🆕 New post inserted: {:?}", message);
            })
            .await
        {
            Ok(subscription_id) => {
                println!("✅ Insert-only subscription created: {}", subscription_id);
            }
            Err(e) => {
                println!("❌ Filtered subscription failed: {}", e);
            }
        }

        println!("\n📋 Realtime Example Complete!");
        println!("💡 To see live updates:");
        println!("   1. Set up a Supabase project");
        println!("   2. Enable Realtime for your table in the dashboard");
        println!("   3. Configure environment variables (SUPABASE_URL, SUPABASE_ANON_KEY)");
        println!("   4. Make changes to your database while this runs");

        println!("\n🎭 Realtime Features Demonstrated:");
        println!("   ✅ WebSocket connection management");
        println!("   ✅ Table change subscriptions");
        println!("   ✅ Event type filtering (INSERT, UPDATE, DELETE)");
    } // End of #[cfg(feature = "realtime")]

    Ok(())
}
