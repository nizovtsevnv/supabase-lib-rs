//! Common types and data structures for Supabase operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Configuration for Supabase client
#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    /// Project URL (e.g., https://your-project.supabase.co)
    pub url: String,
    /// API key (anon key for client-side operations)
    pub key: String,
    /// HTTP client configuration
    pub http_config: HttpConfig,
    /// Auth configuration
    pub auth_config: AuthConfig,
    /// Database configuration
    pub database_config: DatabaseConfig,
    /// Storage configuration
    pub storage_config: StorageConfig,
}

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Request timeout in seconds
    pub timeout: u64,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Maximum number of redirects to follow
    pub max_redirects: usize,
    /// Custom headers to include in all requests
    pub default_headers: HashMap<String, String>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: 60,
            connect_timeout: 10,
            max_redirects: 10,
            default_headers: HashMap::new(),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Auto-refresh tokens before expiry
    pub auto_refresh_token: bool,
    /// Token refresh threshold in seconds before expiry
    pub refresh_threshold: u64,
    /// Persist session in storage
    pub persist_session: bool,
    /// Custom storage implementation
    pub storage_key: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            auto_refresh_token: true,
            refresh_threshold: 300, // 5 minutes
            persist_session: true,
            storage_key: "supabase.auth.token".to_string(),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Default schema to use
    pub schema: String,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            schema: "public".to_string(),
            max_retries: 3,
            retry_delay: 1000,
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Default bucket for operations
    pub default_bucket: Option<String>,
    /// File upload timeout in seconds
    pub upload_timeout: u64,
    /// Maximum file size in bytes
    pub max_file_size: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            default_bucket: None,
            upload_timeout: 300,             // 5 minutes
            max_file_size: 50 * 1024 * 1024, // 50MB
        }
    }
}

/// Generic response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseResponse<T> {
    pub data: Option<T>,
    pub error: Option<SupabaseError>,
}

/// Supabase API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseError {
    pub message: String,
    pub details: Option<String>,
    pub hint: Option<String>,
    pub code: Option<String>,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Current page number (0-based)
    pub page: u32,
    /// Number of items per page
    pub per_page: u32,
    /// Total number of items
    pub total: Option<u64>,
    /// Whether there are more pages
    pub has_more: Option<bool>,
}

/// Query filter operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    #[serde(rename = "eq")]
    Equal,
    #[serde(rename = "neq")]
    NotEqual,
    #[serde(rename = "gt")]
    GreaterThan,
    #[serde(rename = "gte")]
    GreaterThanOrEqual,
    #[serde(rename = "lt")]
    LessThan,
    #[serde(rename = "lte")]
    LessThanOrEqual,
    #[serde(rename = "like")]
    Like,
    #[serde(rename = "ilike")]
    ILike,
    #[serde(rename = "is")]
    Is,
    #[serde(rename = "in")]
    In,
    #[serde(rename = "cs")]
    Contains,
    #[serde(rename = "cd")]
    ContainedBy,
    #[serde(rename = "sl")]
    StrictlyLeft,
    #[serde(rename = "sr")]
    StrictlyRight,
    #[serde(rename = "nxr")]
    NotExtendToRight,
    #[serde(rename = "nxl")]
    NotExtendToLeft,
    #[serde(rename = "adj")]
    Adjacent,
}

/// Order direction for sorting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

/// HTTP method types
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}

/// Generic timestamp type
pub type Timestamp = DateTime<Utc>;

/// Generic ID type
pub type Id = Uuid;

/// JSON value type for dynamic data
pub type JsonValue = serde_json::Value;

/// Headers type for HTTP requests
pub type Headers = HashMap<String, String>;

/// Query parameters type for HTTP requests
pub type QueryParams = HashMap<String, String>;
