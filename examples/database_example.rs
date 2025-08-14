//! Database operations example for Supabase Rust client

use serde::{Deserialize, Serialize};
use std::env;
use supabase::prelude::*;

// Example data structures for database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Post {
    id: Option<i32>,
    title: String,
    content: String,
    author_id: Option<String>,
    published: bool,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreatePost {
    title: String,
    content: String,
    author_id: String,
    published: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdatePost {
    title: Option<String>,
    content: Option<String>,
    published: Option<bool>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let supabase_url =
        env::var("SUPABASE_URL").expect("SUPABASE_URL environment variable is required");
    let supabase_key =
        env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY environment variable is required");

    println!("🗄️ Supabase Rust Client - Database Example");

    let client = Client::new(&supabase_url, &supabase_key)?;
    let database = client.database();

    println!("✅ Client initialized");

    // Example 1: SELECT query with filters and ordering
    println!("\n📖 Example 1: SELECT query with filters");
    let select_query = database
        .from("posts")
        .select("id, title, content, published, created_at")
        .eq("published", "true")
        .order("created_at", OrderDirection::Descending)
        .limit(5);

    match select_query.execute::<Post>().await {
        Ok(posts) => {
            println!("✅ Query executed successfully!");
            println!("   Found {} posts", posts.len());
            for post in posts.iter().take(3) {
                println!(
                    "   - {}: {} ({})",
                    post.id.unwrap_or(0),
                    post.title,
                    if post.published { "Published" } else { "Draft" }
                );
            }
        }
        Err(e) => {
            println!("❌ SELECT query failed: {}", e);
            println!("   This is expected without a 'posts' table in your database");
        }
    }

    // Example 2: Single record query
    println!("\n🔍 Example 2: Single record query");
    let single_query = database.from("posts").select("*").eq("id", "1");

    match single_query.single_execute::<Post>().await {
        Ok(Some(post)) => {
            println!("✅ Found single post:");
            println!("   ID: {}", post.id.unwrap_or(0));
            println!("   Title: {}", post.title);
            println!("   Published: {}", post.published);
        }
        Ok(None) => {
            println!("ℹ️ No post found with ID 1");
        }
        Err(e) => {
            println!("❌ Single query failed: {}", e);
        }
    }

    // Example 3: INSERT operation
    println!("\n➕ Example 3: INSERT operation");
    let new_post = CreatePost {
        title: "Hello from Rust!".to_string(),
        content: "This post was created using the Supabase Rust client.".to_string(),
        author_id: "user_123".to_string(),
        published: false,
    };

    let insert_query = database
        .insert("posts")
        .values(new_post)
        .unwrap()
        .returning("id, title, created_at");

    match insert_query.execute::<Post>().await {
        Ok(inserted_posts) => {
            println!("✅ INSERT successful!");
            if let Some(post) = inserted_posts.first() {
                println!("   Created post ID: {}", post.id.unwrap_or(0));
                println!("   Title: {}", post.title);
                println!(
                    "   Created at: {}",
                    post.created_at
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                );
            }
        }
        Err(e) => {
            println!("❌ INSERT failed: {}", e);
            println!("   This is expected without proper table setup and permissions");
        }
    }

    // Example 4: UPSERT operation
    println!("\n🔄 Example 4: UPSERT operation");
    let upsert_post = CreatePost {
        title: "Updated from Rust!".to_string(),
        content: "This post was upserted using the Supabase Rust client.".to_string(),
        author_id: "user_123".to_string(),
        published: true,
    };

    let upsert_query = database
        .insert("posts")
        .values(upsert_post)
        .unwrap()
        .upsert()
        .on_conflict("author_id")
        .returning("*");

    match upsert_query.execute::<Post>().await {
        Ok(upserted_posts) => {
            println!("✅ UPSERT successful!");
            if let Some(post) = upserted_posts.first() {
                println!("   Post ID: {}", post.id.unwrap_or(0));
                println!("   Title: {}", post.title);
                println!("   Published: {}", post.published);
            }
        }
        Err(e) => {
            println!("❌ UPSERT failed: {}", e);
        }
    }

    // Example 5: UPDATE operation
    println!("\n✏️ Example 5: UPDATE operation");
    let update_data = UpdatePost {
        title: Some("Updated Title from Rust".to_string()),
        content: None,
        published: Some(true),
        updated_at: chrono::Utc::now(),
    };

    let update_query = database
        .update("posts")
        .set(update_data)
        .unwrap()
        .eq("author_id", "user_123")
        .returning("id, title, published, updated_at");

    match update_query.execute::<Post>().await {
        Ok(updated_posts) => {
            println!("✅ UPDATE successful!");
            println!("   Updated {} posts", updated_posts.len());
            for post in updated_posts {
                println!("   - ID {}: {}", post.id.unwrap_or(0), post.title);
            }
        }
        Err(e) => {
            println!("❌ UPDATE failed: {}", e);
        }
    }

    // Example 6: Complex query with multiple filters
    println!("\n🔍 Example 6: Complex query with multiple filters");
    let complex_query = database
        .from("posts")
        .select("id, title, content, published, created_at")
        .eq("published", "true")
        .like("title", "%Rust%")
        .gte("created_at", "2023-01-01T00:00:00Z")
        .order("created_at", OrderDirection::Ascending)
        .limit(10)
        .offset(0);

    match complex_query.execute::<Post>().await {
        Ok(posts) => {
            println!("✅ Complex query executed successfully!");
            println!("   Found {} matching posts", posts.len());
        }
        Err(e) => {
            println!("❌ Complex query failed: {}", e);
        }
    }

    // Example 7: RPC function call
    println!("\n⚙️ Example 7: RPC function call");
    let rpc_params = serde_json::json!({
        "search_term": "Rust",
        "limit": 5
    });

    match database.rpc("search_posts", Some(rpc_params)).await {
        Ok(result) => {
            println!("✅ RPC call successful!");
            println!("   Result: {:?}", result);
        }
        Err(e) => {
            println!("❌ RPC call failed: {}", e);
            println!("   This is expected without the 'search_posts' function defined");
        }
    }

    // Example 8: DELETE operation
    println!("\n🗑️ Example 8: DELETE operation");
    let delete_query = database
        .delete("posts")
        .eq("author_id", "user_123")
        .eq("published", "false")
        .returning("id, title");

    match delete_query.execute::<Post>().await {
        Ok(deleted_posts) => {
            println!("✅ DELETE successful!");
            println!("   Deleted {} posts", deleted_posts.len());
            for post in deleted_posts {
                println!("   - Deleted: {}", post.title);
            }
        }
        Err(e) => {
            println!("❌ DELETE failed: {}", e);
        }
    }

    // Example 9: Query with IN filter
    println!("\n📋 Example 9: Query with IN filter");
    let in_query = database
        .from("posts")
        .select("id, title, author_id")
        .r#in("author_id", &["user_123", "user_456", "user_789"])
        .limit(5);

    match in_query.execute::<Post>().await {
        Ok(posts) => {
            println!("✅ IN query executed successfully!");
            println!("   Found {} posts from specified authors", posts.len());
        }
        Err(e) => {
            println!("❌ IN query failed: {}", e);
        }
    }

    // Example 10: Range queries
    println!("\n📊 Example 10: Range queries");
    let range_query = database
        .from("posts")
        .select("id, title, created_at")
        .gte("created_at", "2023-01-01T00:00:00Z")
        .lt("created_at", "2024-01-01T00:00:00Z")
        .order("created_at", OrderDirection::Descending);

    match range_query.execute::<Post>().await {
        Ok(posts) => {
            println!("✅ Range query executed successfully!");
            println!("   Found {} posts in date range", posts.len());
        }
        Err(e) => {
            println!("❌ Range query failed: {}", e);
        }
    }

    println!("\n✨ Database example completed!");
    println!("💡 To run actual database operations:");
    println!("   1. Set up a Supabase project with a 'posts' table");
    println!("   2. Configure proper authentication and RLS policies");
    println!("   3. Update SUPABASE_URL and SUPABASE_ANON_KEY environment variables");

    Ok(())
}
