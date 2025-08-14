//! Error handling for the Supabase client

use thiserror::Error;

/// Result type alias for Supabase operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for Supabase operations
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP request errors
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

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

    /// Authentication errors
    #[error("Authentication error: {message}")]
    Auth { message: String },

    /// Database operation errors
    #[error("Database error: {message}")]
    Database { message: String },

    /// Storage operation errors
    #[error("Storage error: {message}")]
    Storage { message: String },

    /// Realtime connection errors
    #[error("Realtime error: {message}")]
    Realtime { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Invalid input errors
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// Network errors
    #[error("Network error: {message}")]
    Network { message: String },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String },

    /// Permission denied errors
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    /// Resource not found errors
    #[error("Not found: {message}")]
    NotFound { message: String },

    /// Generic errors
    #[error("{message}")]
    Generic { message: String },

    /// Functions errors
    #[error("Functions error: {message}")]
    Functions { message: String },
}

impl Error {
    /// Create an authentication error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth {
            message: message.into(),
        }
    }

    /// Create a database error
    pub fn database<S: Into<String>>(message: S) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    /// Create a storage error
    pub fn storage<S: Into<String>>(message: S) -> Self {
        Self::Storage {
            message: message.into(),
        }
    }

    /// Create a realtime error
    pub fn realtime<S: Into<String>>(message: S) -> Self {
        Self::Realtime {
            message: message.into(),
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

    /// Create a network error
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limit<S: Into<String>>(message: S) -> Self {
        Self::RateLimit {
            message: message.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied<S: Into<String>>(message: S) -> Self {
        Self::PermissionDenied {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(message: S) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    /// Create a generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Create a functions error
    pub fn functions<T: Into<String>>(message: T) -> Self {
        Self::Functions {
            message: message.into(),
        }
    }
}

/// Handle HTTP status codes and convert to appropriate errors
impl From<reqwest::StatusCode> for Error {
    fn from(status: reqwest::StatusCode) -> Self {
        match status {
            reqwest::StatusCode::UNAUTHORIZED => Self::auth("Unauthorized"),
            reqwest::StatusCode::FORBIDDEN => Self::permission_denied("Forbidden"),
            reqwest::StatusCode::NOT_FOUND => Self::not_found("Resource not found"),
            reqwest::StatusCode::TOO_MANY_REQUESTS => Self::rate_limit("Too many requests"),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => Self::network("Internal server error"),
            reqwest::StatusCode::BAD_GATEWAY => Self::network("Bad gateway"),
            reqwest::StatusCode::SERVICE_UNAVAILABLE => Self::network("Service unavailable"),
            reqwest::StatusCode::GATEWAY_TIMEOUT => Self::network("Gateway timeout"),
            _ => Self::network(format!("HTTP error: {}", status)),
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
    fn test_from_status_code() {
        let error = Error::from(reqwest::StatusCode::NOT_FOUND);
        assert_eq!(error.to_string(), "Not found: Resource not found");
    }
}
