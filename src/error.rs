//! Error handling for the Supabase client

use std::collections::HashMap;
use thiserror::Error;

/// Result type alias for Supabase operations
#[allow(clippy::result_large_err)]
pub type Result<T> = std::result::Result<T, Error>;

/// Platform-specific error context
#[derive(Debug, Clone)]
pub enum PlatformContext {
    /// Native platform (tokio runtime)
    Native {
        /// Operating system information
        os_info: Option<String>,
        /// Available system resources
        system_resources: Option<String>,
    },
    /// WebAssembly platform
    Wasm {
        /// Browser information
        user_agent: Option<String>,
        /// Available Web APIs
        available_apis: Vec<String>,
        /// CORS status
        cors_enabled: bool,
    },
}

/// HTTP error details
#[derive(Debug, Clone)]
pub struct HttpErrorContext {
    /// HTTP status code
    pub status_code: Option<u16>,
    /// Response headers
    pub headers: Option<HashMap<String, String>>,
    /// Response body (if available)
    pub response_body: Option<String>,
    /// Request URL
    pub url: Option<String>,
    /// Request method
    pub method: Option<String>,
}

/// Retry information for failed requests
#[derive(Debug, Clone)]
pub struct RetryInfo {
    /// Number of attempts made
    pub attempts: u32,
    /// Whether the error is retryable
    pub retryable: bool,
    /// Suggested retry delay in seconds
    pub retry_after: Option<u64>,
}

/// Enhanced error context
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Platform-specific context
    pub platform: Option<PlatformContext>,
    /// HTTP error details
    pub http: Option<HttpErrorContext>,
    /// Retry information
    pub retry: Option<RetryInfo>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Error timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            platform: None,
            http: None,
            retry: None,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Main error type for Supabase operations
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP request errors with enhanced context
    #[error("HTTP request failed: {message}")]
    Http {
        message: String,
        #[source]
        source: Option<reqwest::Error>,
        context: ErrorContext,
    },

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// URL parsing errors
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// JWT token errors
    #[cfg(feature = "auth")]
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// Authentication errors with enhanced context
    #[error("Authentication error: {message}")]
    Auth {
        message: String,
        context: ErrorContext,
    },

    /// Database operation errors with enhanced context
    #[error("Database error: {message}")]
    Database {
        message: String,
        context: ErrorContext,
    },

    /// Storage operation errors with enhanced context
    #[error("Storage error: {message}")]
    Storage {
        message: String,
        context: ErrorContext,
    },

    /// Realtime connection errors with enhanced context
    #[error("Realtime error: {message}")]
    Realtime {
        message: String,
        context: ErrorContext,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Invalid input errors
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// Network errors with enhanced context
    #[error("Network error: {message}")]
    Network {
        message: String,
        context: ErrorContext,
    },

    /// Rate limiting errors with retry information
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        context: ErrorContext,
    },

    /// Permission denied errors with enhanced context
    #[error("Permission denied: {message}")]
    PermissionDenied {
        message: String,
        context: ErrorContext,
    },

    /// Resource not found errors with enhanced context
    #[error("Not found: {message}")]
    NotFound {
        message: String,
        context: ErrorContext,
    },

    /// Generic errors
    #[error("{message}")]
    Generic { message: String },

    /// Functions errors with enhanced context
    #[error("Functions error: {message}")]
    Functions {
        message: String,
        context: ErrorContext,
    },

    /// Platform-specific error
    #[error("Platform error: {message}")]
    Platform {
        message: String,
        context: ErrorContext,
    },

    /// Cryptographic error
    #[error("Crypto error: {message}")]
    Crypto {
        message: String,
        context: ErrorContext,
    },
}

impl Error {
    /// Create an authentication error with enhanced context
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create an authentication error with custom context
    pub fn auth_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Auth {
            message: message.into(),
            context,
        }
    }

    /// Create a database error with enhanced context
    pub fn database<S: Into<String>>(message: S) -> Self {
        Self::Database {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a database error with custom context
    pub fn database_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Database {
            message: message.into(),
            context,
        }
    }

    /// Create a storage error with enhanced context
    pub fn storage<S: Into<String>>(message: S) -> Self {
        Self::Storage {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a storage error with custom context
    pub fn storage_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Storage {
            message: message.into(),
            context,
        }
    }

    /// Create a realtime error with enhanced context
    pub fn realtime<S: Into<String>>(message: S) -> Self {
        Self::Realtime {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a realtime error with custom context
    pub fn realtime_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Realtime {
            message: message.into(),
            context,
        }
    }

    /// Create a functions error with enhanced context
    pub fn functions<S: Into<String>>(message: S) -> Self {
        Self::Functions {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a functions error with custom context
    pub fn functions_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Functions {
            message: message.into(),
            context,
        }
    }

    /// Create a network error with enhanced context
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a rate limit error with retry information
    pub fn rate_limit<S: Into<String>>(message: S, retry_after: Option<u64>) -> Self {
        let context = ErrorContext {
            retry: Some(RetryInfo {
                attempts: 0,
                retryable: true,
                retry_after,
            }),
            ..Default::default()
        };

        Self::RateLimit {
            message: message.into(),
            context,
        }
    }

    /// Create a permission denied error with enhanced context
    pub fn permission_denied<S: Into<String>>(message: S) -> Self {
        Self::PermissionDenied {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a not found error with enhanced context
    pub fn not_found<S: Into<String>>(message: S) -> Self {
        Self::NotFound {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create an invalid input error
    pub fn invalid_input<S: Into<String>>(message: S) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    /// Create a generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Get error context if available
    pub fn context(&self) -> Option<&ErrorContext> {
        match self {
            Error::Http { context, .. } => Some(context),
            Error::Auth { context, .. } => Some(context),
            Error::Database { context, .. } => Some(context),
            Error::Storage { context, .. } => Some(context),
            Error::Realtime { context, .. } => Some(context),
            Error::Network { context, .. } => Some(context),
            Error::RateLimit { context, .. } => Some(context),
            Error::PermissionDenied { context, .. } => Some(context),
            Error::NotFound { context, .. } => Some(context),
            Error::Functions { context, .. } => Some(context),
            _ => None,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        self.context()
            .and_then(|ctx| ctx.retry.as_ref())
            .map(|retry| retry.retryable)
            .unwrap_or(false)
    }

    /// Get retry delay in seconds
    pub fn retry_after(&self) -> Option<u64> {
        self.context()
            .and_then(|ctx| ctx.retry.as_ref())
            .and_then(|retry| retry.retry_after)
    }

    /// Get HTTP status code if available
    pub fn status_code(&self) -> Option<u16> {
        self.context()
            .and_then(|ctx| ctx.http.as_ref())
            .and_then(|http| http.status_code)
    }

    /// Create a platform error
    pub fn platform<S: Into<String>>(message: S) -> Self {
        Self::Platform {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a platform error with context
    pub fn platform_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Platform {
            message: message.into(),
            context,
        }
    }

    /// Create a cryptographic error
    pub fn crypto<S: Into<String>>(message: S) -> Self {
        Self::Crypto {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a cryptographic error with context
    pub fn crypto_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Crypto {
            message: message.into(),
            context,
        }
    }
}

/// Detect current platform context
pub fn detect_platform_context() -> PlatformContext {
    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    {
        PlatformContext::Wasm {
            user_agent: web_sys::window().and_then(|window| window.navigator().user_agent().ok()),
            available_apis: detect_available_web_apis(),
            cors_enabled: true, // Assume CORS is enabled for simplicity
        }
    }

    #[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
    {
        PlatformContext::Wasm {
            user_agent: None,
            available_apis: Vec::new(),
            cors_enabled: true,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        PlatformContext::Native {
            os_info: Some(format!(
                "{} {}",
                std::env::consts::OS,
                std::env::consts::ARCH
            )),
            system_resources: None, // Could be enhanced with system info
        }
    }
}

/// Detect available Web APIs in WASM environment
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[allow(dead_code)]
fn detect_available_web_apis() -> Vec<String> {
    let mut apis = Vec::new();

    if let Some(window) = web_sys::window() {
        // Check for common Web APIs
        if window.local_storage().is_ok() {
            apis.push("localStorage".to_string());
        }
        if window.session_storage().is_ok() {
            apis.push("sessionStorage".to_string());
        }
        apis.push("fetch".to_string()); // Fetch API is generally available
    }

    apis
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn detect_available_web_apis() -> Vec<String> {
    Vec::new()
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        let mut context = ErrorContext::default();

        // Add HTTP context if available
        if let Some(status) = err.status() {
            context.http = Some(HttpErrorContext {
                status_code: Some(status.as_u16()),
                headers: None,
                response_body: None,
                url: err.url().map(|u| u.to_string()),
                method: None,
            });

            // Determine if error is retryable
            let retryable = match status.as_u16() {
                500..=599 | 429 | 408 => true, // Server errors, rate limit, timeout
                _ => false,
            };

            context.retry = Some(RetryInfo {
                attempts: 0,
                retryable,
                retry_after: None,
            });
        }

        // Add platform context
        context.platform = Some(detect_platform_context());

        Error::Http {
            message: err.to_string(),
            source: Some(err),
            context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = Error::auth("test message");
        assert_eq!(error.to_string(), "Authentication error: test message");
    }

    #[test]
    fn test_database_error() {
        let error = Error::database("query failed");
        assert_eq!(error.to_string(), "Database error: query failed");
    }

    #[test]
    fn test_error_context() {
        let error = Error::auth("test message");
        assert!(error.context().is_some());
        if let Some(context) = error.context() {
            assert!(context.timestamp <= chrono::Utc::now());
        }
    }
}
