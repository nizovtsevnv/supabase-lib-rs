//! Realtime subscriptions example for Supabase Rust client

use std::{env, sync::Arc, time::Duration};
use supabase_rs::{prelude::*, realtime::RealtimeEvent};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let supabase_url =
        env::var("SUPABASE_URL").expect("SUPABASE_URL environment variable is required");
    let supabase_key =
        env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY environment variable is required");

    println!("⚡ Supabase Rust Client - Realtime Example");

    let client = Client::new(&supabase_url, &supabase_key)?;
    let realtime = client.realtime();

    println!("✅ Client initialized");

    // Example 1: Basic connection test
    println!("\n🔌 Example 1: Test realtime connection");
    println!(
        "Initial connection status: {}",
        if realtime.is_connected().await {
            "✅ Connected"
        } else {
            "❌ Not connected"
        }
    );

    // Example 2: Connect to realtime WebSocket
    println!("\n⚡ Example 2: Connect to realtime");
    match realtime.connect().await {
        Ok(_) => {
            println!("✅ Connected to realtime WebSocket successfully!");

            // Wait a moment for connection to stabilize
            sleep(Duration::from_secs(2)).await;
            println!(
                "Connection status: {}",
                if realtime.is_connected().await {
                    "✅ Connected"
                } else {
                    "❌ Not connected"
                }
            );
        }
        Err(e) => {
            println!("❌ Realtime connection failed: {}", e);
            println!("   This is expected without a properly configured Supabase instance");
            println!("   The following examples will demonstrate API usage patterns");
        }
    }

    // Example 3: Subscribe to all changes on a table
    println!("\n📡 Example 3: Subscribe to table changes");

    let subscription_id = match realtime
        .channel("posts")
        .table("posts")
        .event(RealtimeEvent::All)
        .subscribe(|message| {
            println!("🔄 Received realtime message:");
            println!("   Topic: {}", message.topic);
            println!("   Event: {}", message.event);

            if let Some(payload) = &message.payload.new {
                println!("   New data: {}", payload);
            }

            if let Some(payload) = &message.payload.old {
                println!("   Old data: {}", payload);
            }

            if let Some(schema) = &message.payload.schema {
                println!("   Schema: {}", schema);
            }

            if let Some(table) = &message.payload.table {
                println!("   Table: {}", table);
            }
        })
        .await
    {
        Ok(id) => {
            println!("✅ Subscribed to posts table changes!");
            println!("   Subscription ID: {}", id);
            id
        }
        Err(e) => {
            println!("❌ Subscription failed: {}", e);
            println!("   This is expected without proper realtime setup");
            String::new()
        }
    };

    // Example 4: Subscribe to INSERT events only
    println!("\n➕ Example 4: Subscribe to INSERT events");

    let insert_subscription_id = match realtime
        .channel("posts_inserts")
        .table("posts")
        .event(RealtimeEvent::Insert)
        .subscribe(|message| {
            println!("📝 New post inserted!");
            println!("   Event: {}", message.event);
            if let Some(new_record) = &message.payload.new {
                println!("   Data: {}", new_record);
            }
        })
        .await
    {
        Ok(id) => {
            println!("✅ Subscribed to INSERT events!");
            println!("   Subscription ID: {}", id);
            id
        }
        Err(e) => {
            println!("❌ INSERT subscription failed: {}", e);
            String::new()
        }
    };

    // Example 5: Subscribe to UPDATE events only
    println!("\n✏️ Example 5: Subscribe to UPDATE events");

    let update_subscription_id = match realtime
        .channel("posts_updates")
        .table("posts")
        .event(RealtimeEvent::Update)
        .subscribe(|message| {
            println!("📝 Post updated!");
            println!("   Event: {}", message.event);

            if let Some(old_record) = &message.payload.old {
                println!("   Old data: {}", old_record);
            }

            if let Some(new_record) = &message.payload.new {
                println!("   New data: {}", new_record);
            }
        })
        .await
    {
        Ok(id) => {
            println!("✅ Subscribed to UPDATE events!");
            println!("   Subscription ID: {}", id);
            id
        }
        Err(e) => {
            println!("❌ UPDATE subscription failed: {}", e);
            String::new()
        }
    };

    // Example 6: Subscribe to DELETE events only
    println!("\n🗑️ Example 6: Subscribe to DELETE events");

    let delete_subscription_id = match realtime
        .channel("posts_deletes")
        .table("posts")
        .event(RealtimeEvent::Delete)
        .subscribe(|message| {
            println!("🗑️ Post deleted!");
            println!("   Event: {}", message.event);

            if let Some(old_record) = &message.payload.old {
                println!("   Deleted data: {}", old_record);
            }
        })
        .await
    {
        Ok(id) => {
            println!("✅ Subscribed to DELETE events!");
            println!("   Subscription ID: {}", id);
            id
        }
        Err(e) => {
            println!("❌ DELETE subscription failed: {}", e);
            String::new()
        }
    };

    // Example 7: Subscribe to a different schema
    println!("\n🏗️ Example 7: Subscribe to different schema");

    let schema_subscription_id = match realtime
        .channel("private_posts")
        .schema("private")
        .table("posts")
        .event(RealtimeEvent::All)
        .subscribe(|message| {
            println!("🔒 Private schema change:");
            println!("   Schema: {:?}", message.payload.schema);
            println!("   Table: {:?}", message.payload.table);
            println!("   Event: {}", message.event);
        })
        .await
    {
        Ok(id) => {
            println!("✅ Subscribed to private schema!");
            println!("   Subscription ID: {}", id);
            id
        }
        Err(e) => {
            println!("❌ Schema subscription failed: {}", e);
            String::new()
        }
    };

    // Example 8: Subscribe with filter
    println!("\n🔍 Example 8: Subscribe with filter");

    let filter_subscription_id = match realtime
        .channel("published_posts")
        .table("posts")
        .filter("published=eq.true")
        .event(RealtimeEvent::All)
        .subscribe(|message| {
            println!("📢 Published post changed:");
            println!("   Event: {}", message.event);

            if let Some(new_record) = &message.payload.new {
                println!("   New data: {}", new_record);
            }
        })
        .await
    {
        Ok(id) => {
            println!("✅ Subscribed with filter!");
            println!("   Subscription ID: {}", id);
            println!("   Filter: published=eq.true");
            id
        }
        Err(e) => {
            println!("❌ Filtered subscription failed: {}", e);
            String::new()
        }
    };

    // Example 9: Advanced subscription with custom callback
    println!("\n🎯 Example 9: Advanced subscription with state tracking");

    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);

    let advanced_subscription_id = match realtime
        .channel("tracked_posts")
        .table("posts")
        .event(RealtimeEvent::All)
        .subscribe(move |message| {
            let count = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

            println!("📊 Message #{}: {}", count, message.event);
            println!("   Topic: {}", message.topic);
            println!("   Timestamp: {:?}", message.payload.commit_timestamp);

            // Custom logic based on event type
            match message.event.as_str() {
                "INSERT" => println!("   🆕 A new record was added"),
                "UPDATE" => println!("   ✏️ A record was modified"),
                "DELETE" => println!("   🗑️ A record was removed"),
                _ => println!("   ❓ Unknown event type"),
            }
        })
        .await
    {
        Ok(id) => {
            println!("✅ Advanced subscription created!");
            println!("   Subscription ID: {}", id);
            id
        }
        Err(e) => {
            println!("❌ Advanced subscription failed: {}", e);
            String::new()
        }
    };

    // Example 10: Simulate some activity (in a real app, these would be actual database changes)
    println!("\n⏰ Example 10: Listening for changes");
    println!("   Listening for realtime events for 10 seconds...");
    println!("   In a real application, database changes would trigger these events");

    // Listen for messages for 10 seconds
    for i in 1..=10 {
        println!("   Listening... {} seconds", i);
        sleep(Duration::from_secs(1)).await;

        // In a real application, you might simulate database changes here:
        // client.database().insert("posts").values(new_post).execute().await
    }

    // Example 11: Unsubscribe from channels
    println!("\n🔇 Example 11: Unsubscribe from channels");

    let subscriptions = vec![
        ("Main", subscription_id),
        ("Insert", insert_subscription_id),
        ("Update", update_subscription_id),
        ("Delete", delete_subscription_id),
        ("Schema", schema_subscription_id),
        ("Filter", filter_subscription_id),
        ("Advanced", advanced_subscription_id),
    ];

    for (name, sub_id) in subscriptions {
        if !sub_id.is_empty() {
            match realtime.unsubscribe(&sub_id).await {
                Ok(_) => println!("✅ Unsubscribed from {} subscription", name),
                Err(e) => println!("❌ Failed to unsubscribe from {}: {}", name, e),
            }
        }
    }

    // Example 12: Disconnect from realtime
    println!("\n🔌 Example 12: Disconnect from realtime");
    match realtime.disconnect().await {
        Ok(_) => {
            println!("✅ Disconnected from realtime successfully!");
        }
        Err(e) => {
            println!("❌ Disconnect failed: {}", e);
        }
    }

    // Final connection status
    println!(
        "Final connection status: {}",
        if realtime.is_connected().await {
            "✅ Connected"
        } else {
            "❌ Not connected"
        }
    );

    println!("\n✨ Realtime example completed!");
    println!("💡 To see realtime events in action:");
    println!("   1. Set up a Supabase project with a 'posts' table");
    println!("   2. Enable Realtime for the table in Supabase Dashboard");
    println!("   3. Configure proper authentication and RLS policies");
    println!("   4. Update SUPABASE_URL and SUPABASE_ANON_KEY environment variables");
    println!("   5. Use another client/app to make changes to the database while this runs");
    println!();
    println!("🎭 Realtime Features Demonstrated:");
    println!("   ✅ WebSocket connection management");
    println!("   ✅ Table change subscriptions");
    println!("   ✅ Event type filtering (INSERT, UPDATE, DELETE)");
    println!("   ✅ Schema-specific subscriptions");
    println!("   ✅ Custom filters on subscriptions");
    println!("   ✅ Advanced callback patterns");
    println!("   ✅ Subscription management (subscribe/unsubscribe)");

    Ok(())
}
