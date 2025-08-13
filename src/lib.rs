//! # Supabase Rust Client
//!
//! A comprehensive Rust client library for Supabase that provides:
//! - Authentication (sign up, sign in, JWT handling)
//! - Database operations (CRUD via REST API)
//! - Storage (file upload, download, management)
//! - Realtime subscriptions (WebSocket-based)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use supabase_rs::Client;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct Post {
//!     id: i32,
//!     title: String,
//!     content: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("your-project-url", "your-anon-key")?;
//!
//!     // Authenticate
//!     let _user = client.auth()
//!         .sign_in_with_email_and_password("user@example.com", "password")
//!         .await?;
//!
//!     // Query database with proper type annotation
//!     let _posts: Vec<Post> = client.database()
//!         .from("posts")
//!         .select("*")
//!         .execute()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod database;
pub mod error;
pub mod realtime;
pub mod storage;
pub mod types;

pub use client::Client;
pub use error::{Error, Result};

pub use auth::Auth;
pub use database::Database;
pub use realtime::Realtime;
pub use storage::Storage;

pub mod prelude {
    //! Re-exports of commonly used types and traits

    pub use crate::auth::{Auth, AuthResponse, Session, User};
    pub use crate::database::{
        Database, DeleteBuilder, InsertBuilder, QueryBuilder, UpdateBuilder,
    };
    pub use crate::storage::{Bucket, FileObject, Storage};
    pub use crate::types::{AuthConfig, DatabaseConfig, StorageConfig, SupabaseConfig};
    pub use crate::{Client, Error, Result};
}
