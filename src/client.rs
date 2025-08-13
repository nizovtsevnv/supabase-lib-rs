//! Main Supabase client

use crate::{
    auth::Auth,
    database::Database,
    error::{Error, Result},
    realtime::Realtime,
    storage::Storage,
    types::{AuthConfig, DatabaseConfig, HttpConfig, StorageConfig, SupabaseConfig},
};
use reqwest::{header::HeaderMap, Client as HttpClient};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::{debug, error, info};
use url::Url;

/// Main Supabase client for interacting with all services
#[derive(Debug, Clone)]
pub struct Client {
    /// HTTP client for making requests
    http_client: Arc<HttpClient>,
    /// Client configuration
    config: Arc<SupabaseConfig>,
    /// Authentication module
    auth: Auth,
    /// Database module
    database: Database,
    /// Storage module
    storage: Storage,
    /// Realtime module
    realtime: Realtime,
}

impl Client {
    /// Create a new Supabase client with URL and API key
    ///
    /// # Arguments
    ///
    /// * `url` - Your Supabase project URL (e.g., "https://your-project.supabase.co")
    /// * `key` - Your Supabase API key (anon key for client-side operations)
    ///
    /// # Example
    ///
    /// ```rust
    /// use supabase::Client;
    ///
    /// let client = Client::new("https://your-project.supabase.co", "your-anon-key")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(url: &str, key: &str) -> Result<Self> {
        let config = SupabaseConfig {
            url: url.to_string(),
            key: key.to_string(),
            service_role_key: None,
            http_config: HttpConfig::default(),
            auth_config: AuthConfig::default(),
            database_config: DatabaseConfig::default(),
            storage_config: StorageConfig::default(),
        };

        Self::new_with_config(config)
    }

    /// Create a new Supabase client with service role key for admin operations
    ///
    /// # Arguments
    ///
    /// * `url` - Your Supabase project URL (e.g., "https://your-project.supabase.co")
    /// * `anon_key` - Your Supabase anon API key for client-side operations
    /// * `service_role_key` - Your Supabase service role key for admin operations
    ///
    /// # Example
    ///
    /// ```rust
    /// use supabase::Client;
    ///
    /// let client = Client::new_with_service_role(
    ///     "https://your-project.supabase.co",
    ///     "your-anon-key",
    ///     "your-service-role-key"
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_with_service_role(
        url: &str,
        anon_key: &str,
        service_role_key: &str,
    ) -> Result<Self> {
        let config = SupabaseConfig {
            url: url.to_string(),
            key: anon_key.to_string(),
            service_role_key: Some(service_role_key.to_string()),
            http_config: HttpConfig::default(),
            auth_config: AuthConfig::default(),
            database_config: DatabaseConfig::default(),
            storage_config: StorageConfig::default(),
        };

        Self::new_with_config(config)
    }

    /// Create a new Supabase client with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Custom Supabase configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use supabase::{Client, types::*};
    ///
    /// let config = SupabaseConfig {
    ///     url: "https://your-project.supabase.co".to_string(),
    ///     key: "your-anon-key".to_string(),
    ///     service_role_key: None,
    ///     http_config: HttpConfig::default(),
    ///     auth_config: AuthConfig::default(),
    ///     database_config: DatabaseConfig::default(),
    ///     storage_config: StorageConfig::default(),
    /// };
    ///
    /// let client = Client::new_with_config(config)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_with_config(config: SupabaseConfig) -> Result<Self> {
        // Validate URL
        let _base_url =
            Url::parse(&config.url).map_err(|e| Error::config(format!("Invalid URL: {}", e)))?;

        debug!("Creating Supabase client for URL: {}", config.url);

        // Build HTTP client
        let http_client = Arc::new(Self::build_http_client(&config)?);
        let config = Arc::new(config);

        // Initialize modules
        let auth = Auth::new(Arc::clone(&config), Arc::clone(&http_client))?;
        let database = Database::new(Arc::clone(&config), Arc::clone(&http_client))?;
        let storage = Storage::new(Arc::clone(&config), Arc::clone(&http_client))?;
        let realtime = Realtime::new(Arc::clone(&config))?;

        info!("Supabase client initialized successfully");

        Ok(Self {
            http_client,
            config,
            auth,
            database,
            storage,
            realtime,
        })
    }

    /// Get the authentication module
    pub fn auth(&self) -> &Auth {
        &self.auth
    }

    /// Get the database module
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Get the storage module
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Get the realtime module
    pub fn realtime(&self) -> &Realtime {
        &self.realtime
    }

    /// Get the HTTP client
    pub fn http_client(&self) -> Arc<HttpClient> {
        Arc::clone(&self.http_client)
    }

    /// Get the client configuration
    pub fn config(&self) -> Arc<SupabaseConfig> {
        Arc::clone(&self.config)
    }

    /// Get the base URL for the Supabase project
    pub fn url(&self) -> &str {
        &self.config.url
    }

    /// Get the API key
    pub fn key(&self) -> &str {
        &self.config.key
    }

    /// Set a custom authorization header (JWT token)
    pub async fn set_auth(&self, token: &str) -> Result<()> {
        self.auth.set_session_token(token).await
    }

    /// Clear the current authorization
    pub async fn clear_auth(&self) -> Result<()> {
        self.auth.clear_session().await
    }

    /// Check if client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }

    /// Get current user if authenticated
    pub async fn current_user(&self) -> Result<Option<crate::auth::User>> {
        self.auth.current_user().await
    }

    /// Build HTTP client with configuration
    fn build_http_client(config: &SupabaseConfig) -> Result<HttpClient> {
        let mut headers = HeaderMap::new();

        // Add default headers
        headers.insert(
            "apikey",
            config
                .key
                .parse()
                .map_err(|e| Error::config(format!("Invalid API key: {}", e)))?,
        );
        headers.insert(
            "Authorization",
            format!("Bearer {}", config.key)
                .parse()
                .map_err(|e| Error::config(format!("Invalid authorization header: {}", e)))?,
        );


        // Add custom headers
        for (key, value) in &config.http_config.default_headers {
            let header_name = key
                .parse::<reqwest::header::HeaderName>()
                .map_err(|e| Error::config(format!("Invalid header key '{}': {}", key, e)))?;
            let header_value = value
                .parse::<reqwest::header::HeaderValue>()
                .map_err(|e| Error::config(format!("Invalid header value for '{}': {}", key, e)))?;
            headers.insert(header_name, header_value);
        }

        let client = HttpClient::builder()
            .timeout(Duration::from_secs(config.http_config.timeout))
            .connect_timeout(Duration::from_secs(config.http_config.connect_timeout))
            .redirect(reqwest::redirect::Policy::limited(
                config.http_config.max_redirects,
            ))
            .default_headers(headers)
            .build()
            .map_err(|e| Error::config(format!("Failed to build HTTP client: {}", e)))?;

        Ok(client)
    }

    /// Perform a health check on the Supabase instance
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing health check");

        let response = self
            .http_client
            .get(format!("{}/health", self.config.url))
            .send()
            .await?;

        let is_healthy = response.status().is_success();

        if is_healthy {
            info!("Health check passed");
        } else {
            error!("Health check failed with status: {}", response.status());
        }

        Ok(is_healthy)
    }

    /// Get the current API version information
    pub async fn version(&self) -> Result<HashMap<String, serde_json::Value>> {
        debug!("Fetching version information");

        let response = self
            .http_client
            .get(format!("{}/rest/v1/", self.config.url))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::network(format!(
                "Failed to fetch version info: {}",
                response.status()
            )));
        }

        let version_info = response.json().await?;
        Ok(version_info)
    }
}
