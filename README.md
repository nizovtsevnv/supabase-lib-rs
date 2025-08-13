# Supabase Rust Client

[![CI](https://github.com/nizovtsevnv/supabase-lib-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nizovtsevnv/supabase-lib-rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/supabase-lib-rs)](https://crates.io/crates/supabase-lib-rs)
[![docs.rs](https://docs.rs/supabase-lib-rs/badge.svg)](https://docs.rs/supabase-lib-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Build (musl)](https://github.com/nizovtsevnv/supabase-lib-rs/actions/workflows/build.yml/badge.svg)](https://github.com/nizovtsevnv/supabase-lib-rs/actions/workflows/build.yml)

A comprehensive, production-ready Rust client library for [Supabase](https://supabase.com/). This library provides a clean, type-safe, and efficient interface to interact with all Supabase services.

## 🚀 Features

- **🔐 Authentication** - Complete auth system with JWT handling, user management, and session persistence
- **🗄️ Database** - Type-safe PostgREST API client with query builder pattern
- **📁 Storage** - Full-featured file storage with upload, download, and transformation capabilities
- **⚡ Realtime** - WebSocket subscriptions for live database changes
- **🛡️ Type Safety** - Comprehensive error handling and type definitions
- **🔄 Async/Await** - Full async support with tokio
- **🧪 Well Tested** - Extensive unit and integration test coverage
- **📚 Documentation** - Complete API documentation and examples

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
supabase-lib-rs = "0.1.1"
tokio = { version = "1.0", features = ["full"] }
```

Or use cargo to add it:

```bash
cargo add supabase-lib-rs
```

## 🏃 Quick Start

```rust
use supabase::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client
    let client = Client::new(
        "https://your-project.supabase.co",
        "your-anon-key"
    )?;

    // Or for admin operations with service role key
    let admin_client = Client::new_with_service_role(
        "https://your-project.supabase.co",
        "your-anon-key",
        "your-service-role-key"
    )?;

    // Authenticate user
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
        .eq("published", "true")
        .limit(10)
        .execute::<Post>()
        .await?;

    println!("Found {} posts", posts.len());

    // Upload file to storage
    let upload_result = client
        .storage()
        .upload("avatars", "user-123.jpg", file_bytes, None)
        .await?;

    println!("File uploaded: {}", upload_result.key);

    // Subscribe to realtime changes
    let _subscription_id = client
        .realtime()
        .channel("posts")
        .table("posts")
        .subscribe(|message| {
            println!("Realtime update: {:?}", message);
        })
        .await?;

    Ok(())
}
```

## 📖 Usage Guide

### Authentication

```rust
use supabase::prelude::*;

let client = Client::new("your-url", "your-key")?;

// Sign up new user
let response = client
    .auth()
    .sign_up_with_email_and_password("user@example.com", "password123")
    .await?;

// Sign in existing user
let response = client
    .auth()
    .sign_in_with_email_and_password("user@example.com", "password123")
    .await?;

// Update user profile
let response = client
    .auth()
    .update_user(
        Some("new@example.com".to_string()),
        None,
        Some(serde_json::json!({"display_name": "New Name"}))
    )
    .await?;

// Sign out
client.auth().sign_out().await?;
```

### Database Operations

```rust
use supabase::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: Option<i32>,
    title: String,
    content: String,
    published: bool,
}

let client = Client::new("your-url", "your-key")?;

// SELECT with filters and ordering
let posts = client
    .database()
    .from("posts")
    .select("*")
    .eq("published", "true")
    .gt("created_at", "2023-01-01")
    .order("created_at", OrderDirection::Descending)
    .limit(20)
    .execute::<Post>()
    .await?;

// INSERT new record
let new_post = Post {
    id: None,
    title: "Hello Rust".to_string(),
    content: "Content here".to_string(),
    published: true,
};

let inserted = client
    .database()
    .insert("posts")
    .values(new_post)?
    .returning("*")
    .execute::<Post>()
    .await?;

// UPDATE records
let update_data = serde_json::json!({
    "title": "Updated Title",
    "updated_at": chrono::Utc::now()
});

let updated = client
    .database()
    .update("posts")
    .set(update_data)?
    .eq("id", "123")
    .returning("*")
    .execute::<Post>()
    .await?;

// DELETE records
let deleted = client
    .database()
    .delete("posts")
    .eq("published", "false")
    .returning("id")
    .execute::<Post>()
    .await?;

// Call RPC function
let result = client
    .database()
    .rpc("search_posts", Some(serde_json::json!({
        "search_term": "rust",
        "limit": 10
    })))
    .await?;
```

### Storage Operations

```rust
use supabase::prelude::*;
use bytes::Bytes;

let client = Client::new("your-url", "your-key")?;

// Create bucket
let bucket = client
    .storage()
    .create_bucket("avatars", "User Avatars", true)
    .await?;

// Upload file
let file_content = Bytes::from("Hello, World!");
let upload_result = client
    .storage()
    .upload("avatars", "user-123.txt", file_content, None)
    .await?;

// Upload with options
let options = FileOptions {
    content_type: Some("image/jpeg".to_string()),
    cache_control: Some("max-age=3600".to_string()),
    upsert: true,
};

let upload_result = client
    .storage()
    .upload("avatars", "avatar.jpg", image_bytes, Some(options))
    .await?;

// Download file
let file_data = client
    .storage()
    .download("avatars", "user-123.txt")
    .await?;

// Get public URL
let public_url = client
    .storage()
    .get_public_url("avatars", "avatar.jpg");

// Create signed URL
let signed_url = client
    .storage()
    .create_signed_url("private-files", "document.pdf", 3600)
    .await?;

// List files
let files = client
    .storage()
    .list("avatars", Some("folder/"))
    .await?;
```

### Realtime Subscriptions

```rust
use supabase::prelude::*;

let client = Client::new("your-url", "your-key")?;
let realtime = client.realtime();

// Connect to realtime
realtime.connect().await?;

// Subscribe to all changes on a table
let subscription_id = realtime
    .channel("posts")
    .table("posts")
    .subscribe(|message| {
        println!("Change detected: {}", message.event);

        match message.event.as_str() {
            "INSERT" => println!("New record: {:?}", message.payload.new),
            "UPDATE" => {
                println!("Old: {:?}", message.payload.old);
                println!("New: {:?}", message.payload.new);
            },
            "DELETE" => println!("Deleted: {:?}", message.payload.old),
            _ => {}
        }
    })
    .await?;

// Subscribe to specific events
let insert_subscription = realtime
    .channel("posts_inserts")
    .table("posts")
    .event(RealtimeEvent::Insert)
    .subscribe(|message| {
        println!("New post created!");
    })
    .await?;

// Subscribe with filters
let filtered_subscription = realtime
    .channel("published_posts")
    .table("posts")
    .filter("published=eq.true")
    .subscribe(|message| {
        println!("Published post changed!");
    })
    .await?;

// Unsubscribe
realtime.unsubscribe(&subscription_id).await?;
```

## 🔧 Development

This project uses Nix for reproducible development environments.

### Prerequisites

- [Nix package manager](https://nixos.org/download.html) with flakes enabled
- [just command runner](https://github.com/casey/just) (included in Nix environment)

### Setup

```bash
# Enter the development environment
nix develop

# Or run commands directly
nix develop -c cargo build
```

### Available Commands

```bash
# Show all available commands
just --list

# Format code
just format

# Run linter
just lint

# Run tests
just test

# Build project
just build

# Run all checks (format, lint, test, build)
just check

# Start local Supabase for testing
just supabase-start

# Run examples
just example basic_usage
just example auth_example
just example database_example
just example storage_example
just example realtime_example
```

### Project Structure

```
supabase-lib-rs/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── client.rs       # Main Supabase client
│   ├── auth.rs         # Authentication module
│   ├── database.rs     # Database operations
│   ├── storage.rs      # File storage
│   ├── realtime.rs     # WebSocket subscriptions
│   ├── error.rs        # Error handling
│   └── types.rs        # Common types and configurations
├── examples/           # Usage examples
├── tests/             # Integration tests
├── flake.nix          # Nix development environment
├── justfile           # Command runner configuration
└── CLAUDE.md          # Development guidelines
```

## 🧪 Testing

```bash
# Run unit tests
just test-unit

# Run integration tests (requires Supabase setup)
just test-integration

# Run with coverage
just coverage
```

### Integration Tests

To run integration tests, you need a Supabase instance:

1. Set up a local Supabase instance:

   ```bash
   just supabase-start
   ```

2. Or configure environment variables for your Supabase project:

   ```bash
   export SUPABASE_URL="https://your-project.supabase.co"
   export SUPABASE_ANON_KEY="your-anon-key"
   ```

3. Run the tests:
   ```bash
   just test-integration
   ```

## 📚 Examples

The `examples/` directory contains comprehensive examples:

- **`basic_usage.rs`** - Overview of all features
- **`auth_example.rs`** - Authentication flows
- **`database_example.rs`** - Database operations
- **`storage_example.rs`** - File storage operations
- **`realtime_example.rs`** - WebSocket subscriptions

Run examples with:

```bash
cargo run --example basic_usage
```

## ⚙️ Configuration

### Environment Variables

The library can be configured using environment variables. Copy `.env.example` to `.env` and fill in your actual values:

```bash
cp .env.example .env
```

**Required variables:**

- `SUPABASE_URL` - Your Supabase project URL
- `SUPABASE_ANON_KEY` - Your Supabase anonymous key

**Optional variables:**

- `SUPABASE_SERVICE_ROLE_KEY` - Service role key for admin operations
- `RUST_LOG` - Log level (debug, info, warn, error)
- `RUST_BACKTRACE` - Enable backtrace (0, 1, full)

### Getting Your Supabase Keys

1. Go to your [Supabase Dashboard](https://supabase.com/dashboard)
2. Select your project
3. Navigate to Settings > API
4. Copy the keys:
   - **Project URL** → `SUPABASE_URL`
   - **anon public** → `SUPABASE_ANON_KEY`
   - **service_role** → `SUPABASE_SERVICE_ROLE_KEY` (keep this secret!)

### Custom Configuration

```rust
use supabase::{Client, types::*};

let config = SupabaseConfig {
    url: "https://your-project.supabase.co".to_string(),
    key: "your-anon-key".to_string(),
    service_role_key: None,
    http_config: HttpConfig {
        timeout: 30,
        connect_timeout: 10,
        max_redirects: 5,
        default_headers: HashMap::new(),
    },
    auth_config: AuthConfig {
        auto_refresh_token: true,
        refresh_threshold: 300,
        persist_session: true,
        storage_key: "supabase.auth.token".to_string(),
    },
    database_config: DatabaseConfig {
        schema: "public".to_string(),
        max_retries: 3,
        retry_delay: 1000,
    },
    storage_config: StorageConfig {
        default_bucket: Some("uploads".to_string()),
        upload_timeout: 300,
        max_file_size: 50 * 1024 * 1024,
    },
};

let client = Client::new_with_config(config)?;
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes following the coding standards
4. Run the full test suite: `just check`
5. Submit a pull request

### Code Standards

- Follow the existing code style
- Add tests for new features
- Update documentation as needed
- Ensure all checks pass: `just check`

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [Supabase Documentation](https://supabase.com/docs)
- [PostgREST API Reference](https://postgrest.org/en/latest/api.html)
- [Supabase Realtime](https://supabase.com/docs/guides/realtime)
- [Crates.io Package](https://crates.io/crates/supabase-lib-rs)
- [Documentation](https://docs.rs/supabase-lib-rs)

## 🙏 Acknowledgments

- [Supabase](https://supabase.com/) for providing an amazing backend platform
- The Rust community for excellent crates and tooling
- Contributors who help improve this library

---

**Made with ❤️ for the Rust and Supabase communities**
