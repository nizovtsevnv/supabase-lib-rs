//! # Supabase Rust Client
//!
//! A comprehensive Rust client library for [Supabase](https://supabase.com) that provides
//! full-featured access to all Supabase services with excellent cross-platform support.
//!
//! ## Features
//!
//! - **🔐 Authentication**: Sign up, sign in, JWT handling, session management
//! - **🗄️ Database**: Full REST API access with query builder, CRUD operations, RPC
//! - **📁 Storage**: File upload, download, management with bucket operations
//! - **⚡ Realtime**: WebSocket subscriptions for live database changes
//! - **🌍 Cross-platform**: Works on native (tokio) and WASM (browser) targets
//! - **🛡️ Type-safe**: Full type safety with serde integration
//! - **⚙️ Configurable**: Feature flags for minimal builds
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! supabase-lib-rs = "0.2.0"
//! tokio = { version = "1.0", features = ["full"] }
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! ### Basic Usage
//!
//! ```rust,no_run
//! use supabase::Client;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct Post {
//!     id: Option<i32>,
//!     title: String,
//!     content: String,
//!     author_id: i32,
//!     created_at: Option<String>,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the client
//!     let client = Client::new("your-project-url", "your-anon-key")?;
//!
//!     // Authentication
//!     let auth_response = client.auth()
//!         .sign_in_with_email_and_password("user@example.com", "password")
//!         .await?;
//!
//!     println!("Signed in: {:?}", auth_response.user);
//!
//!     // Database operations
//!     let posts: Vec<Post> = client.database()
//!         .from("posts")
//!         .select("*")
//!         .eq("author_id", "123")
//!         .order("created_at", supabase::types::OrderDirection::Descending)
//!         .limit(10)
//!         .execute()
//!         .await?;
//!
//!     println!("Found {} posts", posts.len());
//!
//!     // Insert new post
//!     let new_post = Post {
//!         id: None,
//!         title: "My First Post".to_string(),
//!         content: "Hello, Supabase!".to_string(),
//!         author_id: 123,
//!         created_at: None,
//!     };
//!
//!     let created: Vec<Post> = client.database()
//!         .insert("posts")
//!         .values(new_post)?
//!         .returning("*")
//!         .execute()
//!         .await?;
//!
//!     println!("Created post: {:?}", created.first());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! Enable only the features you need to reduce bundle size:
//!
//! ```toml
//! [dependencies]
//! supabase-lib-rs = { version = "0.2.0", features = ["auth", "database"] }
//! ```
//!
//! Available features:
//! - `auth` - Authentication and user management
//! - `database` - Database operations and query builder
//! - `storage` - File storage operations
//! - `realtime` - WebSocket subscriptions
//! - `native` - Native tokio runtime (default)
//! - `wasm` - WebAssembly support for browsers
//!
//! ## Platform Support
//!
//! ### Native Applications
//!
//! For desktop, server, and mobile applications using tokio:
//!
//! ```rust,no_run
//! # #[cfg(feature = "realtime")]
//! # async fn example() -> supabase::Result<()> {
//! use supabase::Client;
//!
//! let client = Client::new("your-url", "your-key")?;
//!
//! // Full feature support including realtime
//! let realtime = client.realtime();
//! realtime.connect().await?;
//!
//! let subscription = realtime
//!     .channel("posts")
//!     .table("posts")
//!     .subscribe(|msg| println!("Update: {:?}", msg))
//!     .await?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ### WASM/Browser Applications
//!
//! For web applications running in browsers:
//!
//! ```toml
//! [dependencies]
//! supabase-lib-rs = { version = "0.2.0", features = ["wasm", "auth", "database", "storage"] }
//! wasm-bindgen = "0.2"
//! wasm-bindgen-futures = "0.4"
//! ```
//!
//! ```rust,no_run
//! use supabase::Client;
//! use wasm_bindgen::prelude::*;
//!
//! // In your WASM entry point:
//! let client = Client::new("your-url", "your-key").unwrap();
//!
//! wasm_bindgen_futures::spawn_local(async move {
//!     // Auth works in browser
//!     match client.auth().sign_in_with_email_and_password("user@example.com", "password").await {
//!         Ok(_response) => web_sys::console::log_1(&"Signed in!".into()),
//!         Err(e) => web_sys::console::log_1(&format!("Error: {}", e).into()),
//!     }
//! });
//! ```
//!
//! ## Advanced Examples
//!
//! ### Database Query Builder
//!
//! Build complex queries with the fluent API:
//!
//! ```rust,no_run
//! # #[cfg(feature = "database")]
//! # async fn example() -> supabase::Result<()> {
//! use supabase::{Client, types::OrderDirection};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize)]
//! struct User {
//!     id: i32,
//!     email: String,
//!     name: Option<String>,
//!     active: bool,
//! }
//!
//! let client = Client::new("your-url", "your-key")?;
//!
//! // Complex query with filters, ordering, and pagination
//! let active_users: Vec<User> = client.database()
//!     .from("users")
//!     .select("id, email, name, active")
//!     .eq("active", "true")
//!     .ilike("email", "%@company.com")
//!     .order("name", OrderDirection::Ascending)
//!     .limit(50) // Limit to 50 results
//!     .execute()
//!     .await?;
//!
//! // Update with conditions - using a struct for updates
//! use std::collections::HashMap;
//! let mut update_data = HashMap::new();
//! update_data.insert("last_seen", "now()");
//!
//! let _updated: Vec<User> = client.database()
//!     .update("users")
//!     .set(update_data)?
//!     .eq("id", "123")
//!     .execute()
//!     .await?;
//!
//! // Delete with conditions
//! let _deleted: Vec<User> = client.database()
//!     .delete("users")
//!     .eq("active", "false")
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### File Storage Operations
//!
//! ```rust,no_run
//! # #[cfg(feature = "storage")]
//! # async fn example() -> supabase::Result<()> {
//! use supabase::Client;
//!
//! let client = Client::new("your-url", "your-key")?;
//! let storage = client.storage();
//!
//! // Create a bucket
//! storage.create_bucket("avatars", "avatars", true).await?; // public bucket
//!
//! // Upload a file
//! let file_content = b"Hello, world!";
//! let upload_response = storage
//!     .upload("avatars", "user123/avatar.jpg", file_content.to_vec().into(), None)
//!     .await?;
//!
//! println!("Uploaded: {}", upload_response.key);
//!
//! // Download a file
//! let file_data = storage
//!     .download("avatars", "user123/avatar.jpg")
//!     .await?;
//!
//! // Get public URL
//! let public_url = storage.get_public_url("avatars", "user123/avatar.jpg");
//! println!("Public URL: {}", public_url);
//! # Ok(())
//! # }
//! ```
//!
//! ### Realtime Subscriptions
//!
//! Listen to database changes in real-time:
//!
//! ```rust,no_run
//! # #[cfg(feature = "realtime")]
//! # async fn example() -> supabase::Result<()> {
//! use supabase::Client;
//! # #[cfg(feature = "realtime")]
//! use supabase::realtime::RealtimeEvent;
//!
//! let client = Client::new("your-url", "your-key")?;
//! let realtime = client.realtime();
//!
//! // Connect to realtime
//! realtime.connect().await?;
//!
//! // Subscribe to all changes on a table
//! let sub1 = realtime
//!     .channel("all-posts")
//!     .table("posts")
//!     .subscribe(|message| {
//!         println!("Posts table changed: {:?}", message.event);
//!         if let Some(record) = &message.payload.record {
//!             println!("New data: {}", record);
//!         }
//!     })
//!     .await?;
//!
//! // Subscribe to specific events with filters
//! let sub2 = realtime
//!     .channel("user-posts")
//!     .table("posts")
//!     .event(RealtimeEvent::Insert) // Only new posts
//!     .filter("author_id=eq.123")   // Only from specific author
//!     .subscribe(|message| {
//!         println!("New post from author 123!");
//!     })
//!     .await?;
//!
//! // Later, unsubscribe
//! realtime.unsubscribe(&sub1).await?;
//! realtime.unsubscribe(&sub2).await?;
//! realtime.disconnect().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Error Handling
//!
//! The library provides comprehensive error types:
//!
//! ```rust,no_run
//! use supabase::{Client, Error};
//!
//! # async fn example() {
//! let client = Client::new("your-url", "your-key").unwrap();
//!
//! match client.auth().sign_in_with_email_and_password("user@example.com", "wrong").await {
//!     Ok(response) => println!("Success: {:?}", response.user),
//!     Err(Error::Auth { message }) => println!("Auth error: {}", message),
//!     Err(Error::Network { message }) => println!("Network error: {}", message),
//!     Err(Error::Http(e)) => println!("HTTP error: {}", e),
//!     Err(e) => println!("Other error: {}", e),
//! }
//! # }
//! ```
//!
//! ## Configuration
//!
//! Customize the client behavior:
//!
//! ```rust,no_run
//! use supabase::{Client, types::*};
//!
//! let config = SupabaseConfig {
//!     url: "your-url".to_string(),
//!     key: "your-key".to_string(),
//!     http_config: HttpConfig {
//!         timeout: 30,           // 30 second timeout
//!         connect_timeout: 5,    // 5 second connect timeout
//!         max_redirects: 3,      // Maximum 3 redirects
//!         default_headers: std::collections::HashMap::new(),
//!     },
//!     auth_config: AuthConfig {
//!         auto_refresh_token: true,  // Auto-refresh JWT tokens
//!         refresh_threshold: 600,    // Refresh 10 minutes before expiry
//!         persist_session: true,     // Persist session in storage
//!         storage_key: "custom.auth.token".to_string(),
//!     },
//!     database_config: DatabaseConfig {
//!         schema: "public".to_string(),
//!         max_retries: 3,
//!         retry_delay: 1000,
//!     },
//!     storage_config: StorageConfig {
//!         upload_timeout: 300,                // 5 minute upload timeout
//!         max_file_size: 100 * 1024 * 1024,   // 100MB max file size
//!         default_bucket: Some("uploads".to_string()),
//!     },
//!     service_role_key: None,
//! };
//!
//! let client = Client::new_with_config(config).unwrap();
//! ```
//!
//! ## Migration from JavaScript SDK
//!
//! This library follows similar patterns to the JavaScript SDK:
//!
//! | JavaScript | Rust |
//! |------------|------|
//! | `supabase.auth.signInWithEmailAndPassword()` | `client.auth().sign_in_with_email_and_password()` |
//! | `supabase.from('table').select()` | `client.database().from("table").select()` |
//! | `supabase.storage.from('bucket').upload()` | `client.storage().upload("bucket", ...)` |
//! | `supabase.channel().on().subscribe()` | `client.realtime().channel().subscribe()` |
//!
//! ## Examples Repository
//!
//! Check out the [examples](https://github.com/your-repo/supabase-lib-rs/tree/main/examples)
//! directory for complete applications:
//!
//! - `basic_usage.rs` - Overview of all features
//! - `auth_example.rs` - Authentication flows
//! - `database_example.rs` - Complex database operations
//! - `storage_example.rs` - File management
//! - `realtime_example.rs` - Live subscriptions
//! - `dioxus_example.rs` - Full-stack web app
//! - `wasm_example.rs` - Browser integration
//!
//! ## Contributing
//!
//! We welcome contributions! Please see [CONTRIBUTING.md](https://github.com/your-repo/supabase-lib-rs/blob/main/CONTRIBUTING.md)
//! for guidelines.
//!
//! ## License
//!
//! This project is licensed under the MIT License - see the [LICENSE](https://github.com/your-repo/supabase-lib-rs/blob/main/LICENSE)
//! file for details.

#[cfg(feature = "auth")]
pub mod auth;

pub mod client;

#[cfg(feature = "database")]
pub mod database;

pub mod error;

#[cfg(feature = "realtime")]
pub mod realtime;

#[cfg(feature = "storage")]
pub mod storage;

#[cfg(feature = "functions")]
pub mod functions;

#[cfg(feature = "ffi")]
pub mod ffi;

pub mod types;

// Internal modules
#[cfg(feature = "realtime")]
mod async_runtime;

#[cfg(feature = "realtime")]
mod websocket;

pub use client::Client;
pub use error::{Error, Result};

#[cfg(feature = "auth")]
pub use auth::Auth;

#[cfg(feature = "database")]
pub use database::Database;

#[cfg(feature = "realtime")]
pub use realtime::Realtime;

#[cfg(feature = "storage")]
pub use storage::Storage;

#[cfg(feature = "functions")]
pub use functions::Functions;

/// Commonly used types and traits for convenient importing
///
/// This module re-exports the most frequently used types to make them
/// easily accessible with a single import.
///
/// ## Usage
///
/// ```rust
/// use supabase::prelude::*;
///
/// // Now you have access to Client, Error, Result, and all common types
/// # fn example() -> Result<()> {
/// let client = Client::new("url", "key")?;
/// # Ok(())
/// # }
/// ```
pub mod prelude {

    pub use crate::types::*;
    pub use crate::{Client, Error, Result};

    #[cfg(feature = "auth")]
    pub use crate::auth::{Auth, AuthResponse, Session, User};

    #[cfg(feature = "database")]
    pub use crate::database::{
        Database, DeleteBuilder, InsertBuilder, QueryBuilder, UpdateBuilder,
    };

    #[cfg(feature = "storage")]
    pub use crate::storage::{Bucket, FileObject, Storage};

    #[cfg(feature = "functions")]
    pub use crate::functions::Functions;

    #[cfg(feature = "realtime")]
    pub use crate::realtime::{Realtime, RealtimeEvent, RealtimeMessage};
}
