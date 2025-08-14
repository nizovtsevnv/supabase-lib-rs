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
- **🌐 WASM Support** - Full WebAssembly compatibility for web applications
- **🦀 Cross-Platform** - Works on native (desktop/server) and WASM (web) targets
- **🧪 Well Tested** - Extensive unit and integration test coverage
- **📚 Documentation** - Complete API documentation and examples

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
supabase-lib-rs = "0.2.0"
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

This project has a comprehensive testing system with multiple levels of testing:

### Unit Tests

```bash
# Run unit tests only
just test-unit

# Run with documentation tests
just test
```

### Integration & E2E Tests

```bash
# Start local Supabase (requires Docker/Podman)
just supabase-start

# Run integration tests
just test-integration

# Run all tests (unit + doc + integration)
just test-all
```

### Docker/Podman Setup

The project includes a complete local Supabase setup using Docker Compose:

```bash
# Start all Supabase services
just supabase-start

# Check status
just supabase-status

# View logs
just supabase-logs [service]

# Stop services
just supabase-stop

# Clean up data
just supabase-clean
```

**Services provided:**

- 🌐 **Studio**: http://localhost:54323 (Web UI)
- 🔗 **API**: http://localhost:54321 (REST + Auth + Realtime)
- 🗄️ **Database**: localhost:54322 (PostgreSQL)
- 📁 **Storage**: File storage with image processing
- ⚡ **Functions**: Edge functions runtime

### Test Categories

1. **Unit Tests** - Fast, isolated component tests
2. **Integration Tests** - Test individual modules against real Supabase
3. **E2E Tests** - Full workflow scenarios
4. **Doc Tests** - Ensure documentation examples work

All integration tests automatically skip if Supabase is not available, making them safe for CI/CD.

## 🚧 Current Limitations

While this library provides comprehensive Supabase functionality, some advanced features are planned for future releases:

### Authentication

- **OAuth Providers**: Google, GitHub, Discord, Apple, etc. (planned for v0.3.0)
- **Phone Authentication**: SMS OTP and phone number sign-in (planned for v0.3.0)
- **Multi-Factor Authentication (MFA)**: TOTP and SMS-based 2FA (planned for v0.4.0)
- **Auth Events**: `onAuthStateChange` event listeners (planned for v0.3.0)
- **Anonymous Sign-in**: Temporary anonymous user sessions (planned for v0.3.0)

### Database

- **Logical Operators**: Complex `and()`, `or()`, `not()` query logic (planned for v0.3.0)
- **Full-Text Search**: `textSearch()` and advanced search operators (planned for v0.4.0)
- **Query Analysis**: `explain()` and CSV export functionality (planned for v0.4.0)

### Missing Modules

- ~~**Edge Functions**: `functions.invoke()` for serverless functions~~ ✅ **Added in v0.3.0**
- **Management API**: Project management and admin operations (planned for v0.4.0)

### Workarounds

Most limitations can be worked around:

```rust
// Instead of OAuth, use magic links or email/password
let auth_response = client.auth()
    .sign_up_with_email_and_password("user@example.com", "password")
    .await?;

// Instead of logical operators, use multiple queries or raw SQL
let result = client.database()
    .rpc("custom_query", Some(json!({"param": "value"})))
    .await?;

// Instead of Edge Functions, use database RPC functions
let function_result = client.database()
    .rpc("my_custom_function", Some(params))
    .await?;
```

The library currently provides **~90% of core Supabase functionality** and covers all common use cases for production applications.

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

## 🌐 WebAssembly (WASM) Support

This library provides full WebAssembly support for web applications! You can use it with frameworks like [Dioxus](https://dioxuslabs.com/), [Yew](https://yew.rs/), or any WASM-based Rust web framework.

### WASM Features

- **✅ Full API compatibility** - Same API works on both native and WASM
- **✅ HTTP client** - Uses browser's fetch API automatically
- **✅ Authentication** - Complete auth flow support
- **✅ Database** - All CRUD operations and query builder
- **✅ Storage** - File upload/download (simplified for WASM)
- **✅ Realtime** - WebSocket subscriptions via browser WebSocket API

### Building for WASM

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Build for WASM
cargo build --target wasm32-unknown-unknown

# Build example for WASM
cargo build --target wasm32-unknown-unknown --example wasm_example
```

### WASM Example

```rust
use supabase::{Client, Result};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() {
    let client = Client::new("your-url", "your-key").unwrap();

    // Works exactly the same as native!
    let todos = client
        .database()
        .from("todos")
        .select("*")
        .execute::<Todo>()
        .await
        .unwrap();

    web_sys::console::log_1(&format!("Got {} todos", todos.len()).into());
}
```

### Integration with Web Frameworks

**Dioxus:**

```rust
use dioxus::prelude::*;
use supabase::Client;

fn App(cx: Scope) -> Element {
    let client = use_state(cx, || {
        Client::new("your-url", "your-key").unwrap()
    });

    // Use client in your components...
}
```

**Yew:**

```rust
use yew::prelude::*;
use supabase::Client;

#[function_component(App)]
fn app() -> Html {
    let client = use_state(|| {
        Client::new("your-url", "your-key").unwrap()
    });

    // Use client in your components...
}
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
