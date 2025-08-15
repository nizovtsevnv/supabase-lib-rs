# Usage Examples

This document provides comprehensive examples for all Supabase Rust Client features.

## Quick Start

```rust
use supabase::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client
    let client = Client::new(
        "https://your-project.supabase.co",
        "your-anon-key"
    )?;

    // Sign in a user
    let auth_response = client
        .auth()
        .sign_in_with_email_and_password("user@example.com", "password")
        .await?;

    println!("User signed in: {:?}", auth_response.user);

    // Query database
    let posts = client
        .database()
        .from("posts")
        .select("id, title, content")
        .limit(10)
        .execute()
        .await?;

    println!("Posts: {:?}", posts);

    Ok(())
}
```

## Authentication Examples

### Email/Password Authentication

```rust
use supabase::prelude::*;

async fn email_auth_example() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    // Sign up
    let signup_response = client
        .auth()
        .sign_up_with_email_and_password("user@example.com", "password")
        .await?;
    
    // Sign in
    let signin_response = client
        .auth()
        .sign_in_with_email_and_password("user@example.com", "password")
        .await?;

    // Get current user
    if let Some(user) = client.auth().current_user().await {
        println!("Current user: {}", user.email);
    }

    // Sign out
    client.auth().sign_out().await?;
    
    Ok(())
}
```

### OAuth Authentication

```rust
async fn oauth_example() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    // Get OAuth URL
    let auth_url = client
        .auth()
        .get_oauth_url("google", "https://yourapp.com/callback")
        .await?;
    
    println!("Visit: {}", auth_url);

    // After OAuth callback, exchange code for session
    let session = client
        .auth()
        .exchange_oauth_code("authorization_code")
        .await?;
    
    Ok(())
}
```

### Multi-Factor Authentication

```rust
async fn mfa_example() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    // Sign in first
    client.auth()
        .sign_in_with_email_and_password("user@example.com", "password")
        .await?;

    // Setup TOTP MFA
    let mfa_setup = client.auth().setup_mfa().await?;
    println!("Scan QR code: {}", mfa_setup.qr_code);

    // Verify TOTP code
    let verification = client.auth()
        .verify_mfa("123456")
        .await?;

    println!("MFA verified: {}", verification.success);

    Ok(())
}
```

## Database Examples

### Basic Queries

```rust
async fn database_queries() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    // Select with filters
    let posts = client
        .database()
        .from("posts")
        .select("id, title, author, created_at")
        .eq("status", "published")
        .order("created_at", Some(false))
        .limit(20)
        .execute()
        .await?;

    // Insert data
    let new_post = serde_json::json!({
        "title": "My New Post",
        "content": "This is the content",
        "author_id": 123
    });

    let inserted = client
        .database()
        .from("posts")
        .insert(new_post)
        .execute()
        .await?;

    // Update data
    let updated = client
        .database()
        .from("posts")
        .update(serde_json::json!({"status": "published"}))
        .eq("id", "1")
        .execute()
        .await?;

    // Delete data
    client
        .database()
        .from("posts")
        .delete()
        .eq("id", "1")
        .execute()
        .await?;

    Ok(())
}
```

### Advanced Queries

```rust
async fn advanced_queries() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    // Complex filters
    let results = client
        .database()
        .from("posts")
        .select("*, author(*), comments(*)")
        .or("status.eq.published,featured.eq.true")
        .gt("view_count", 100)
        .in_("category", vec!["tech", "programming"])
        .execute()
        .await?;

    // Full-text search
    let search_results = client
        .database()
        .from("posts")
        .select("*")
        .fts("title", "rust programming")
        .execute()
        .await?;

    // Aggregations
    let stats = client
        .database()
        .from("posts")
        .select("count(*), avg(view_count)")
        .execute()
        .await?;

    Ok(())
}
```

### Transactions

```rust
async fn transaction_example() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    let transaction = client.database().begin_transaction().await?;

    // Multiple operations in transaction
    let user_id = transaction
        .from("users")
        .insert(serde_json::json!({"email": "user@example.com"}))
        .execute()
        .await?;

    transaction
        .from("profiles")
        .insert(serde_json::json!({
            "user_id": user_id,
            "display_name": "New User"
        }))
        .execute()
        .await?;

    // Commit transaction
    transaction.commit().await?;

    Ok(())
}
```

## Storage Examples

### File Operations

```rust
async fn storage_operations() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    // Upload file
    let file_data = std::fs::read("path/to/file.jpg")?;
    let upload_result = client
        .storage()
        .from("avatars")
        .upload("user-123/avatar.jpg", file_data)
        .await?;

    println!("File uploaded: {}", upload_result.path);

    // Download file
    let downloaded = client
        .storage()
        .from("avatars")
        .download("user-123/avatar.jpg")
        .await?;

    // Get public URL
    let public_url = client
        .storage()
        .from("avatars")
        .get_public_url("user-123/avatar.jpg");

    // Delete file
    client
        .storage()
        .from("avatars")
        .remove(&["user-123/avatar.jpg"])
        .await?;

    Ok(())
}
```

### Large File Upload

```rust
async fn large_file_upload() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    // Progress callback
    let progress_callback = |bytes_uploaded: usize, total_bytes: usize| {
        let percentage = (bytes_uploaded as f64 / total_bytes as f64) * 100.0;
        println!("Upload progress: {:.1}%", percentage);
    };

    // Upload with progress
    let result = client
        .storage()
        .from("documents")
        .upload_large_file(
            "presentation.pptx",
            "path/to/large-file.pptx",
            Some(Box::new(progress_callback))
        )
        .await?;

    println!("Large file uploaded: {}", result.path);

    Ok(())
}
```

## Functions Examples

### Edge Function Invocation

```rust
async fn functions_example() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    // Simple function call
    let response = client
        .functions()
        .invoke("hello-world", serde_json::json!({"name": "World"}))
        .await?;

    println!("Function response: {:?}", response);

    // Function with custom headers
    let mut headers = std::collections::HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "value".to_string());

    let response = client
        .functions()
        .invoke_with_options(
            "process-data",
            serde_json::json!({"data": "example"}),
            Some(headers),
            Some(30) // timeout in seconds
        )
        .await?;

    Ok(())
}
```

## Realtime Examples

### Database Subscriptions

```rust
async fn realtime_subscriptions() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    // Subscribe to table changes
    let subscription = client
        .realtime()
        .channel("posts")
        .on("postgres_changes", |payload| {
            println!("Database change: {:?}", payload);
        })
        .subscribe()
        .await?;

    // Listen for specific events
    subscription
        .on("INSERT", |payload| {
            println!("New post inserted: {:?}", payload);
        })
        .on("UPDATE", |payload| {
            println!("Post updated: {:?}", payload);
        })
        .on("DELETE", |payload| {
            println!("Post deleted: {:?}", payload);
        });

    // Keep connection alive
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

    // Unsubscribe
    subscription.unsubscribe().await?;

    Ok(())
}
```

### Presence System

```rust
async fn presence_example() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    let channel = client
        .realtime()
        .channel("room:lobby")
        .subscribe()
        .await?;

    // Track user presence
    channel.track(serde_json::json!({
        "user_id": 123,
        "username": "alice",
        "status": "online"
    })).await?;

    // Listen for presence changes
    channel.on("presence", |event| {
        match event.event_type.as_str() {
            "sync" => println!("Initial presence: {:?}", event.payload),
            "join" => println!("User joined: {:?}", event.payload),
            "leave" => println!("User left: {:?}", event.payload),
            _ => {}
        }
    });

    Ok(())
}
```

### Broadcasting

```rust
async fn broadcast_example() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    let channel = client
        .realtime()
        .channel("chat:general")
        .subscribe()
        .await?;

    // Listen for broadcast messages
    channel.on("broadcast", |event| {
        if event.event_type == "message" {
            println!("New message: {:?}", event.payload);
        }
    });

    // Send broadcast message
    channel.send("message", serde_json::json!({
        "user": "alice",
        "text": "Hello everyone!"
    })).await?;

    Ok(())
}
```

## Error Handling

### Comprehensive Error Handling

```rust
use supabase::{Error, AuthError, DatabaseError};

async fn error_handling_example() -> Result<()> {
    let client = Client::new("your-url", "your-key")?;

    match client.auth().sign_in_with_email_and_password("user@example.com", "wrong-password").await {
        Ok(response) => {
            println!("Signed in successfully");
        },
        Err(Error::Auth(AuthError::InvalidCredentials)) => {
            println!("Invalid email or password");
        },
        Err(Error::Auth(AuthError::UserNotConfirmed)) => {
            println!("Please check your email and confirm your account");
        },
        Err(Error::Network { source, .. }) => {
            println!("Network error: {}", source);
        },
        Err(e) => {
            println!("Other error: {}", e);
        }
    }

    Ok(())
}
```

## WASM Examples

### Browser Integration

```rust
use supabase::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn initialize_supabase() -> Result<(), JsValue> {
    let client = Client::new(
        "https://your-project.supabase.co",
        "your-anon-key"
    ).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Auth with browser storage
    let auth_response = client
        .auth()
        .sign_in_with_email_and_password("user@example.com", "password")
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    web_sys::console::log_1(&format!("Signed in: {:?}", auth_response.user).into());

    Ok(())
}
```

## Best Practices

### Connection Management

```rust
use std::sync::Arc;

// Share client instance across your application
#[derive(Clone)]
pub struct AppState {
    pub supabase: Arc<Client>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let client = Client::new("your-url", "your-key")?;
        Ok(Self {
            supabase: Arc::new(client)
        })
    }
}

// Use in handlers
async fn create_post(state: AppState, post_data: PostData) -> Result<()> {
    let result = state.supabase
        .database()
        .from("posts")
        .insert(serde_json::to_value(post_data)?)
        .execute()
        .await?;

    Ok(())
}
```

### Retry Logic

```rust
use tokio::time::{sleep, Duration};

async fn retry_operation<T, F, Fut>(
    operation: F,
    max_retries: u32,
    delay: Duration,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempts = 0;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                println!("Attempt {} failed, retrying...", attempts);
                sleep(delay).await;
            },
            Err(e) => return Err(e),
        }
    }
}

// Usage
let result = retry_operation(
    || client.database().from("posts").select("*").execute(),
    3,
    Duration::from_secs(1),
).await?;
```

## Related Documentation

- [Configuration Guide](CONFIGURATION.md)
- [Architecture Guide](ARCHITECTURE.md) 
- [WebAssembly Guide](WASM_GUIDE.md)
- [API Documentation](https://docs.rs/supabase-lib-rs) 