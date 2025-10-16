//! Storage operations example for Supabase Rust client

use bytes::Bytes;
use std::env;
use supabase_lib_rs::prelude::*;

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let supabase_url =
        env::var("SUPABASE_URL").expect("SUPABASE_URL environment variable is required");
    let supabase_key =
        env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY environment variable is required");

    println!("📁 Supabase Rust Client - Storage Example");

    let client = Client::new(&supabase_url, &supabase_key)?;
    let storage = client.storage();

    println!("✅ Client initialized");

    // Example 1: List all storage buckets
    println!("\n📋 Example 1: List storage buckets");
    match storage.list_buckets().await {
        Ok(buckets) => {
            println!("✅ Found {} buckets", buckets.len());
            for bucket in buckets.iter().take(3) {
                println!(
                    "   - {}: {} ({})",
                    bucket.id,
                    bucket.name,
                    if bucket.public { "Public" } else { "Private" }
                );
            }
        }
        Err(e) => {
            println!("❌ List buckets failed: {}", e);
            println!("   This is expected without proper Supabase Storage setup");
        }
    }

    // Example 2: Create a new bucket
    let test_bucket_id = "test-bucket-rust";
    println!("\n🪣 Example 2: Create storage bucket");
    match storage
        .create_bucket(test_bucket_id, "Test Bucket from Rust", true)
        .await
    {
        Ok(bucket) => {
            println!("✅ Created bucket successfully!");
            println!("   Bucket ID: {}", bucket.id);
            println!("   Bucket Name: {}", bucket.name);
            println!("   Public: {}", bucket.public);
            println!("   Created: {}", bucket.created_at);
        }
        Err(e) => {
            println!("❌ Create bucket failed: {}", e);
            println!(
                "   This is expected if the bucket already exists or without proper permissions"
            );
        }
    }

    // Example 3: Get bucket information
    println!("\n🔍 Example 3: Get bucket information");
    match storage.get_bucket(test_bucket_id).await {
        Ok(bucket) => {
            println!("✅ Retrieved bucket info:");
            println!("   ID: {}", bucket.id);
            println!("   Name: {}", bucket.name);
            println!("   Public: {}", bucket.public);
            println!("   File size limit: {:?}", bucket.file_size_limit);
        }
        Err(e) => {
            println!("❌ Get bucket failed: {}", e);
        }
    }

    // Example 4: Upload file from bytes
    println!("\n⬆️ Example 4: Upload file from bytes");
    let file_content = "Hello, Supabase Storage from Rust!\nThis is a test file.";
    let file_bytes = Bytes::from(file_content.as_bytes());
    let file_path = "test/hello.txt";

    let upload_options = supabase_lib_rs::storage::FileOptions {
        content_type: Some("text/plain".to_string()),
        cache_control: Some("max-age=3600".to_string()),
        upsert: true,
    };

    match storage
        .upload(test_bucket_id, file_path, file_bytes, Some(upload_options))
        .await
    {
        Ok(response) => {
            println!("✅ File uploaded successfully!");
            println!("   Key: {}", response.key);
            println!("   ID: {:?}", response.id);
        }
        Err(e) => {
            println!("❌ File upload failed: {}", e);
            println!("   This is expected without proper bucket and permissions setup");
        }
    }

    // Example 5: List files in bucket
    println!("\n📂 Example 5: List files in bucket");
    match storage.list(test_bucket_id, Some("test/")).await {
        Ok(files) => {
            println!("✅ Found {} files in test/ folder", files.len());
            for file in files.iter().take(5) {
                println!("   - {}", file.name);
                println!("     Created: {:?}", file.created_at);
                println!("     Updated: {:?}", file.updated_at);
            }
        }
        Err(e) => {
            println!("❌ List files failed: {}", e);
        }
    }

    // Example 6: Download file
    println!("\n⬇️ Example 6: Download file");
    match storage.download(test_bucket_id, file_path).await {
        Ok(downloaded_bytes) => {
            println!("✅ File downloaded successfully!");
            println!("   Size: {} bytes", downloaded_bytes.len());

            // Try to convert to string if it's text
            if let Ok(content) = String::from_utf8(downloaded_bytes.to_vec()) {
                println!(
                    "   Content preview: {}",
                    content.chars().take(100).collect::<String>()
                );
            }
        }
        Err(e) => {
            println!("❌ File download failed: {}", e);
        }
    }

    // Example 7: Get public URL
    println!("\n🔗 Example 7: Get public URL");
    let public_url = storage.get_public_url(test_bucket_id, file_path);
    println!("✅ Public URL generated:");
    println!("   {}", public_url);

    // Example 8: Create signed URL for private access
    println!("\n🔐 Example 8: Create signed URL");
    match storage
        .create_signed_url(test_bucket_id, file_path, 3600, None) // 1 hour, no transforms
        .await
    {
        Ok(signed_url) => {
            println!("✅ Signed URL created (expires in 1 hour):");
            println!("   {}", signed_url);
        }
        Err(e) => {
            println!("❌ Create signed URL failed: {}", e);
        }
    }

    // Example 9: Upload file with image transformations URL
    println!("\n🖼️ Example 9: Image transformations (example URL)");
    let image_path = "images/avatar.jpg";

    let transform_options = supabase_lib_rs::storage::TransformOptions {
        width: Some(200),
        height: Some(200),
        resize: Some(supabase_lib_rs::storage::ResizeMode::Cover),
        format: Some(supabase_lib_rs::storage::ImageFormat::Webp),
        quality: Some(80),
    };

    match storage.get_public_url_transformed(test_bucket_id, image_path, transform_options) {
        Ok(transformed_url) => {
            println!("✅ Transformed image URL:");
            println!("   {}", transformed_url);
            println!("   (200x200, cover resize, WebP format, 80% quality)");
        }
        Err(e) => {
            println!("❌ Generate transformed URL failed: {}", e);
        }
    }

    // Example 10: Copy file
    let copied_file_path = "test/hello_copy.txt";
    println!("\n📋 Example 10: Copy file");
    match storage
        .copy(test_bucket_id, file_path, copied_file_path)
        .await
    {
        Ok(_) => {
            println!("✅ File copied successfully!");
            println!("   From: {}", file_path);
            println!("   To: {}", copied_file_path);
        }
        Err(e) => {
            println!("❌ File copy failed: {}", e);
        }
    }

    // Example 11: Move file
    let moved_file_path = "test/moved/hello.txt";
    println!("\n🚚 Example 11: Move file");
    match storage
        .r#move(test_bucket_id, copied_file_path, moved_file_path)
        .await
    {
        Ok(_) => {
            println!("✅ File moved successfully!");
            println!("   From: {}", copied_file_path);
            println!("   To: {}", moved_file_path);
        }
        Err(e) => {
            println!("❌ File move failed: {}", e);
        }
    }

    // Example 12: Upload file from filesystem (demo)
    println!("\n💾 Example 12: Upload from filesystem (demo)");
    // Create a temporary file for demonstration
    let temp_file_path = "/tmp/supabase_test.txt";
    if std::fs::write(temp_file_path, "Test content from filesystem").is_ok() {
        let file_content = match std::fs::read(temp_file_path) {
            Ok(content) => content,
            Err(e) => {
                println!("❌ Failed to read temp file: {}", e);
                return Ok(());
            }
        };

        match storage
            .upload(
                test_bucket_id,
                "uploads/from_fs.txt",
                file_content.into(),
                None,
            )
            .await
        {
            Ok(response) => {
                println!("✅ File uploaded from filesystem!");
                println!("   Key: {}", response.key);
            }
            Err(e) => {
                println!("❌ Upload from filesystem failed: {}", e);
            }
        }

        // Clean up temp file
        let _ = std::fs::remove_file(temp_file_path);
    } else {
        println!("⚠️ Could not create temporary file for demo");
    }

    // Example 13: Delete files
    println!("\n🗑️ Example 13: Delete files");
    let files_to_delete = &[file_path, moved_file_path, "uploads/from_fs.txt"];

    match storage.remove(test_bucket_id, files_to_delete).await {
        Ok(_) => {
            println!("✅ Files deleted successfully!");
            for file_path in files_to_delete {
                println!("   - {}", file_path);
            }
        }
        Err(e) => {
            println!("❌ File deletion failed: {}", e);
        }
    }

    // Example 14: Update bucket settings
    println!("\n⚙️ Example 14: Update bucket settings");
    match storage.update_bucket(test_bucket_id, Some(false)).await {
        Ok(_) => {
            println!("✅ Bucket updated to private!");
        }
        Err(e) => {
            println!("❌ Bucket update failed: {}", e);
        }
    }

    // Example 15: Delete bucket (cleanup)
    println!("\n🧹 Example 15: Delete bucket (cleanup)");
    match storage.delete_bucket(test_bucket_id).await {
        Ok(_) => {
            println!("✅ Test bucket deleted successfully!");
        }
        Err(e) => {
            println!("❌ Bucket deletion failed: {}", e);
            println!("   You may need to manually clean up the test bucket");
        }
    }

    println!("\n✨ Storage example completed!");
    println!("💡 To run actual storage operations:");
    println!("   1. Set up a Supabase project with Storage enabled");
    println!("   2. Configure proper authentication and RLS policies");
    println!("   3. Update SUPABASE_URL and SUPABASE_ANON_KEY environment variables");
    println!("   4. Ensure proper bucket permissions for file operations");

    Ok(())
}
