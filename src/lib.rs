#![allow(clippy::result_large_err)]

//! # Supabase Rust Client Library
//!
//! A comprehensive, production-ready Rust client library for Supabase with full cross-platform support (native + WASM).
//!
//! ## 🚀 Version 0.3.1 - Enhanced Authentication
//!
//! This release introduces a comprehensive authentication system with OAuth providers,
//! phone authentication, magic links, anonymous sign-in, and real-time auth state events,
//! plus advanced cross-platform error handling with detailed context and retry logic.
//!
//! ## ✨ Features
//!
//! - **🔐 Authentication**: Complete auth system with OAuth, phone, magic links, anonymous sign-in
//! - **🗄️ Database**: Advanced PostgreSQL operations with joins, transactions, logical operators
//! - **📁 Storage**: File upload, download, and management
//! - **⚡ Realtime**: WebSocket subscriptions for live data
//! - **🔧 Functions**: Edge Functions invocation
//! - **🌐 Cross-Platform**: Native (Tokio) and WASM support
//! - **🛡️ Type Safe**: Full type safety with comprehensive error handling
//! - **📚 Well Documented**: Extensive documentation with examples
//!
//! ## 🚀 Quick Start
//!
//! ```rust
//! use supabase::auth::AuthEvent;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct User {
//!     id: i32,
//!     name: String,
//!     email: String,
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = supabase::Client::new("https://example.supabase.co", "valid-key")?;
//!
//!     // Set up authentication event listener
//!     let _handle = client.auth().on_auth_state_change(|event, session| {
//!         match event {
//!             AuthEvent::SignedIn => println!("User signed in!"),
//!             AuthEvent::SignedOut => println!("User signed out!"),
//!             AuthEvent::TokenRefreshed => println!("Token refreshed!"),
//!             _ => {}
//!         }
//!     });
//!
//!     // Sign up a new user
//!     let auth_response = client.auth()
//!         .sign_up_with_email_and_password("user@example.com", "secure_password")
//!         .await?;
//!
//!     println!("User created: {:?}", auth_response.user);
//!
//!     // Complex database query with JOIN
//!     let posts_with_users: Vec<serde_json::Value> = client.database()
//!         .from("posts")
//!         .select("title, content, users(name, email)")
//!         .inner_join("users", "name, email")
//!         .eq("published", "true")
//!         .execute()
//!         .await?;
//!
//!     println!("Found {} posts with users", posts_with_users.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## 🔐 Authentication Examples
//!
//! ### OAuth Providers
//!
//! ```rust
//! use supabase::auth::{OAuthProvider, OAuthOptions};
//!
//! # async fn example() -> supabase::Result<()> {
//! # let client = supabase::Client::new("https://example.supabase.co", "your-anon-key")?;
//! // Google OAuth
//! let options = OAuthOptions {
//!     redirect_to: Some("https://myapp.com/callback".to_string()),
//!     scopes: Some(vec!["email".to_string(), "profile".to_string()]),
//!     ..Default::default()
//! };
//!
//! let response = client.auth()
//!     .sign_in_with_oauth(OAuthProvider::Google, Some(options))
//!     .await?;
//!
//! println!("Redirect to: {}", response.url);
//! # Ok(())
//! # }
//! ```
//!
//! ### Phone Authentication
//!
//! ```rust
//! # async fn example() -> supabase::Result<()> {
//! # let client = supabase::Client::new("https://example.supabase.co", "your-anon-key")?;
//! // Sign up with phone
//! let auth_response = client.auth()
//!     .sign_up_with_phone("+1234567890", "secure_password", None)
//!     .await?;
//!
//! // Verify OTP
//! let verified = client.auth()
//!     .verify_otp("+1234567890", "123456", "sms")
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Magic Links
//!
//! ```rust
//! # async fn example() -> supabase::Result<()> {
//! # let client = supabase::Client::new("https://example.supabase.co", "your-anon-key")?;
//! // Send magic link
//! client.auth()
//!     .sign_in_with_magic_link(
//!         "user@example.com",
//!         Some("https://myapp.com/callback".to_string()),
//!         None
//!     )
//!     .await?;
//!
//! println!("Magic link sent!");
//! # Ok(())
//! # }
//! ```
//!
//! ### Anonymous Sign-in
//!
//! ```rust
//! # async fn example() -> supabase::Result<()> {
//! # let client = supabase::Client::new("https://example.supabase.co", "your-anon-key")?;
//! // Create anonymous user
//! let auth_response = client.auth()
//!     .sign_in_anonymously(None)
//!     .await?;
//!
//! if let Some(user) = auth_response.user {
//!     println!("Anonymous user created: {}", user.id);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## 🗄️ Advanced Database Operations
//!
//! ### Complex Queries
//!
//! ```rust,no_run
//! # use supabase::Client;
//! # use supabase::types::OrderDirection;
//! # use serde_json::Value;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let client = supabase::Client::new("https://example.supabase.co", "your-anon-key")?;
//!   let posts: Vec<Value> = client.database()
//!       .from("posts")
//!       .select("*")
//!       .and(|q| q.eq("published", "true").gte("created_at", "2024-01-01"))
//!       .or(|q| q.eq("author", "admin").eq("status", "featured"))
//!       .not(|q| q.eq("deleted", "true"))
//!       .order("created_at", OrderDirection::Descending)
//!       .limit(10)
//!       .execute()
//!       .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Query Joins
//!
//! ```rust,no_run
//! # use supabase::Client;
//! # use serde_json::Value;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let client = supabase::Client::new("https://example.supabase.co", "your-anon-key")?;
//!   let posts_with_authors: Vec<Value> = client.database()
//!       .from("posts")
//!       .select("id, title, users(name, email)")
//!       .inner_join_as("users", "name", "author_name")
//!       .execute()
//!       .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Transactions
//!
//! ```rust,no_run
//! # use supabase::Client;
//! # use serde::{Deserialize, Serialize};
//! # use serde_json::json;
//! #
//! # #[derive(Debug, Deserialize, Serialize)]
//! # struct User { id: i32, name: String, email: String }
//! #
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let client = supabase::Client::new("https://example.supabase.co", "your-anon-key")?;
//!   let result: Vec<User> = client.database()
//!       .begin_transaction()
//!       .insert("users", json!({"name": "John", "email": "john@example.com"}))
//!       .update("profiles", json!({"updated_at": "now()"}), "user_id = $1")
//!       .commit()
//!       .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 🌐 Cross-Platform Support
//!
//! This library works seamlessly across different platforms:
//!
//! ### Native Applications (Tokio)
//! ```rust
//! // Full async/await support with Tokio runtime
//! #[tokio::main]
//! async fn main() -> supabase::Result<()> {
//!     let client = supabase::Client::new("https://example.supabase.co", "your-anon-key")?;
//!     // All operations available
//!     Ok(())
//! }
//! ```
//!
//! ### WebAssembly (WASM)
//! ```rust
//! use wasm_bindgen::prelude::*;
//!
//! #[wasm_bindgen]
//! pub async fn initialize_supabase() -> Result<(), JsValue> {
//!     let client = supabase::Client::new("https://example.supabase.co", "your-anon-key")
//!         .map_err(|e| JsValue::from_str(&e.to_string()))?;
//!
//!     // Enhanced error handling with platform-specific context
//!     match client.database().from("users").select("*").execute::<serde_json::Value>().await {
//!         Ok(users) => web_sys::console::log_1(&format!("Found {} users", users.len()).into()),
//!         Err(e) => {
//!             let error_msg = format!("Database error: {}", e);
//!             web_sys::console::error_1(&error_msg.clone().into());
//!             return Err(JsValue::from_str(&error_msg));
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## 🛡️ Enhanced Error Handling
//!
//! v0.3.1 introduces comprehensive error context with platform-specific information:
//!
//! ```rust
//! use supabase::error::{ErrorContext, PlatformContext};
//!
//! # async fn example() -> supabase::Result<()> {
//! # let client = supabase::Client::new("https://example.supabase.co", "your-anon-key")?;
//! match client.auth().sign_in_with_email_and_password("user@example.com", "password").await {
//!     Ok(response) => println!("Success!"),
//!     Err(e) => {
//!         // Check if error is retryable
//!         if e.is_retryable() {
//!             if let Some(retry_after) = e.retry_after() {
//!                 println!("Retry after {} seconds", retry_after);
//!             }
//!         }
//!
//!         // Get platform-specific context
//!         if let Some(context) = e.context() {
//!             match &context.platform {
//!                 Some(PlatformContext::Wasm { user_agent, available_apis, .. }) => {
//!                     println!("WASM environment: {:?}", user_agent);
//!                     println!("Available APIs: {:?}", available_apis);
//!                 }
//!                 Some(PlatformContext::Native { os_info, .. }) => {
//!                     println!("Native environment: {:?}", os_info);
//!                 }
//!                 None => {}
//!             }
//!         }
//!
//!         // Get HTTP status code if available
//!         if let Some(status) = e.status_code() {
//!             println!("HTTP Status: {}", status);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## 📚 Module Organization
//!
//! - [`auth`] - Authentication operations and OAuth providers
//! - [`database`] - PostgreSQL database operations with advanced querying
//! - [`storage`] - File upload, download, and management
//! - [`realtime`] - WebSocket subscriptions for live data
//! - [`functions`] - Edge Functions invocation
//! - [`error`] - Enhanced error types with platform-specific context
//! - [`types`] - Common type definitions and configurations
//!
//! ## 🔧 Configuration
//!
//! ```rust,no_run
//! use supabase::{Client, types::{SupabaseConfig, HttpConfig, AuthConfig}};
//! use std::time::Duration;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::new("YOUR_URL", "YOUR_KEY")?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 🚧 Version History
//!
//! - **v0.3.1**: Enhanced Authentication with OAuth, phone auth, magic links, anonymous sign-in, improved error context
//! - **v0.3.0**: Advanced Database Operations with joins, transactions, logical operators, C FFI foundation
//! - **v0.2.0**: Production-ready client with comprehensive testing and documentation
//!
//! ## 📄 License
//!
//! This project is licensed under the MIT License.

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "session-management")]
pub mod session;

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
/// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
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
