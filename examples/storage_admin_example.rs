//! Storage admin operations example with service role key

use std::env;
use supabase_lib_rs::prelude::*;

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let supabase_url =
        env::var("SUPABASE_URL").expect("SUPABASE_URL environment variable is required");
    let supabase_anon_key =
        env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY environment variable is required");
    let supabase_service_key = env::var("SUPABASE_SERVICE_ROLE_KEY")
        .expect("SUPABASE_SERVICE_ROLE_KEY environment variable is required");

    println!("🔐 Supabase Rust Client - Storage Admin Example");

    // Create client with service role key for admin operations
    let client =
        Client::new_with_service_role(&supabase_url, &supabase_anon_key, &supabase_service_key)?;
    let storage = client.storage();

    println!("✅ Client initialized with service role key");

    // Test bucket creation with admin privileges
    println!("\n🪣 Example 1: Create storage bucket with admin key");
    let bucket_name = "test-admin-bucket";
    match storage
        .create_bucket(bucket_name, "Admin Test Bucket", true)
        .await
    {
        Ok(bucket) => {
            println!("✅ Bucket created successfully!");
            println!("   ID: {}", bucket.id);
            println!("   Name: {}", bucket.name);
            println!("   Public: {}", bucket.public);
        }
        Err(e) => {
            println!("❌ Bucket creation failed: {}", e);
            println!("   This may be expected if bucket already exists");
        }
    }

    // Test bucket listing
    println!("\n📋 Example 2: List all buckets");
    match storage.list_buckets().await {
        Ok(buckets) => {
            println!("✅ Found {} buckets:", buckets.len());
            for bucket in buckets.iter().take(3) {
                println!(
                    "   - {}: {} ({})",
                    bucket.id,
                    bucket.name,
                    if bucket.public { "public" } else { "private" }
                );
            }
        }
        Err(e) => {
            println!("❌ List buckets failed: {}", e);
        }
    }

    // Cleanup - try to delete the test bucket
    println!("\n🧹 Example 3: Delete test bucket");
    match storage.delete_bucket(bucket_name).await {
        Ok(_) => {
            println!("✅ Test bucket deleted successfully");
        }
        Err(e) => {
            println!("❌ Bucket deletion failed: {}", e);
            println!("   You may need to manually clean up: {}", bucket_name);
        }
    }

    println!("\n✨ Storage admin example completed!");
    Ok(())
}
