//! Storage with authentication example
//!
//! This example demonstrates how to use authenticated storage operations
//! with Row Level Security (RLS) policies in Supabase.
//!
//! To run this example:
//! ```bash
//! cargo run --example storage_auth_example
//! ```

use bytes::Bytes;
use std::env;
use supabase_lib_rs::prelude::*;

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> supabase_lib_rs::Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Get credentials from environment
    let supabase_url = env::var("SUPABASE_URL").expect("SUPABASE_URL must be set");
    let supabase_key = env::var("SUPABASE_KEY").expect("SUPABASE_KEY must be set");
    let user_email = env::var("USER_EMAIL").expect("USER_EMAIL must be set");
    let user_password = env::var("USER_PASSWORD").expect("USER_PASSWORD must be set");

    // Create Supabase client
    let client = Client::new(&supabase_url, &supabase_key)?;

    println!("🔐 Signing in user...");

    // Sign in to get user authentication token
    let auth_response = client
        .auth()
        .sign_in_with_email_and_password(&user_email, &user_password)
        .await?;

    let session = auth_response
        .session
        .ok_or_else(|| supabase_lib_rs::Error::auth("No session returned".to_string()))?;

    let user = auth_response
        .user
        .ok_or_else(|| supabase_lib_rs::Error::auth("No user returned".to_string()))?;

    let user_token = session.access_token.clone();
    let user_id = user.id;
    println!("✅ User signed in successfully");
    println!("   User ID: {}", user_id);
    println!(
        "   Email: {}",
        user.email.as_ref().unwrap_or(&"N/A".to_string())
    );

    // Example bucket and file paths
    let bucket_id = "private-files"; // Your private bucket with RLS policies
    let file_path = format!("users/{}/document.txt", user_id);

    println!("\n📤 Uploading file to protected bucket...");

    // Upload file with user authentication
    // This will work if RLS policy allows authenticated users to upload to their own folder
    let file_content = Bytes::from("This is a private document for the authenticated user.");

    match client
        .storage()
        .upload_with_auth(
            bucket_id,
            &file_path,
            file_content.clone(),
            None,
            Some(&user_token),
        )
        .await
    {
        Ok(response) => {
            println!("✅ File uploaded successfully!");
            println!("   Key: {}", response.key);
        }
        Err(e) => {
            eprintln!("❌ Upload failed: {}", e);
            eprintln!("   Make sure your RLS policy allows uploads for authenticated users");
        }
    }

    println!("\n📂 Listing files in protected bucket...");

    // List files with user authentication
    match client
        .storage()
        .list_with_auth(
            bucket_id,
            Some(&format!("users/{}/", user_id)),
            Some(&user_token),
        )
        .await
    {
        Ok(files) => {
            println!("✅ Found {} files:", files.len());
            for file in files.iter().take(5) {
                println!("   - {}", file.name);
            }
        }
        Err(e) => {
            eprintln!("❌ List failed: {}", e);
        }
    }

    println!("\n📥 Downloading file from protected bucket...");

    // Download file with user authentication
    match client
        .storage()
        .download_with_auth(bucket_id, &file_path, Some(&user_token))
        .await
    {
        Ok(file_data) => {
            println!("✅ File downloaded successfully!");
            println!("   Size: {} bytes", file_data.len());
            println!("   Content: {}", String::from_utf8_lossy(&file_data));
        }
        Err(e) => {
            eprintln!("❌ Download failed: {}", e);
        }
    }

    println!("\n🗑️  Deleting file from protected bucket...");

    // Delete file with user authentication
    match client
        .storage()
        .remove_with_auth(bucket_id, &[&file_path], Some(&user_token))
        .await
    {
        Ok(()) => {
            println!("✅ File deleted successfully!");
        }
        Err(e) => {
            eprintln!("❌ Delete failed: {}", e);
        }
    }

    // Comparison: Operations without authentication token (will fail with RLS)
    println!("\n⚠️  Attempting operations WITHOUT authentication token...");
    println!("   (These should fail if RLS policies are properly configured)");

    match client
        .storage()
        .upload(
            bucket_id,
            "test-unauthenticated.txt",
            Bytes::from("test"),
            None,
        )
        .await
    {
        Ok(_) => println!("   ⚠️  Upload without auth succeeded (RLS may not be configured)"),
        Err(e) => println!("   ✅ Upload without auth failed as expected: {}", e),
    }

    println!("\n✨ Example complete!");
    println!("\n💡 Tips for using authenticated storage:");
    println!("   1. Set up Row Level Security (RLS) policies in Supabase dashboard");
    println!("   2. Use *_with_auth() methods when working with protected buckets");
    println!("   3. Pass the user's session.access_token for authenticated operations");
    println!("   4. RLS policies can restrict access based on user ID, role, etc.");

    Ok(())
}
