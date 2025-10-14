//! Edge Functions module for Supabase
//!
//! This module provides functionality to invoke Supabase Edge Functions.
//! Edge Functions are server-side TypeScript functions that run on the edge,
//! close to your users for reduced latency.
//!
//! ## Features
//!
//! - **Standard Invocation**: Traditional request/response function calls
//! - **Streaming Responses**: Server-sent events and streaming data
//! - **Function Metadata**: Introspection and function discovery
//! - **Local Development**: Testing utilities for local functions
//! - **Enhanced Error Handling**: Detailed error context and retry logic

use crate::{
    error::{Error, Result},
    types::SupabaseConfig,
};
use reqwest::Client as HttpClient;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
#[cfg(not(target_arch = "wasm32"))]
use tokio_stream::Stream;
use tracing::{debug, info, warn};

// Helper for async sleep across platforms
#[cfg(not(target_arch = "wasm32"))]
async fn async_sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
async fn async_sleep(duration: Duration) {
    use gloo_timers::future::sleep as gloo_sleep;
    gloo_sleep(duration).await;
}

#[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
async fn async_sleep(_duration: Duration) {
    // No-op for wasm32 without wasm feature (retry delays not supported)
}

/// Edge Functions client for invoking serverless functions
///
/// # Examples
///
/// Basic function invocation:
///
/// ```rust,no_run
/// use supabase::Client;
/// use serde_json::json;
///
/// # async fn example() -> supabase::Result<()> {
/// let client = Client::new("your-project-url", "your-anon-key")?;
///
/// // Invoke a function with parameters
/// let result = client.functions()
///     .invoke("hello-world", Some(json!({"name": "World"})))
///     .await?;
///
/// println!("Function result: {}", result);
/// # Ok(())
/// # }
/// ```
///
/// Streaming function responses:
///
/// ```rust,no_run
/// use supabase::Client;
/// use serde_json::json;
/// use tokio_stream::StreamExt;
///
/// # async fn example() -> supabase::Result<()> {
/// let client = Client::new("your-project-url", "your-anon-key")?;
///
/// // Stream function responses
/// let mut stream = client.functions()
///     .invoke_stream("data-processor", Some(json!({"batch_size": 100})))
///     .await?;
///
/// while let Some(chunk) = stream.next().await {
///     match chunk {
///         Ok(data) => println!("Received: {:?}", data),
///         Err(e) => println!("Stream error: {}", e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Functions {
    http_client: Arc<HttpClient>,
    config: Arc<SupabaseConfig>,
}

/// Function metadata and introspection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// Function name
    pub name: String,
    /// Function description
    pub description: Option<String>,
    /// Function version
    pub version: Option<String>,
    /// Runtime environment
    pub runtime: Option<String>,
    /// Memory limit in MB
    pub memory_limit: Option<u32>,
    /// Timeout in seconds
    pub timeout: Option<u32>,
    /// Environment variables (non-sensitive)
    pub env_vars: HashMap<String, String>,
    /// Function status
    pub status: FunctionStatus,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last modified timestamp
    pub updated_at: Option<String>,
}

/// Function execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FunctionStatus {
    /// Function is active and can be invoked
    Active,
    /// Function is paused/disabled
    Inactive,
    /// Function is deploying
    Deploying,
    /// Function deployment failed
    Failed,
}

/// Configuration for function invocation
#[derive(Debug, Clone, Default)]
pub struct InvokeOptions {
    /// Additional headers to send
    pub headers: Option<HashMap<String, String>>,
    /// Function timeout override
    pub timeout: Option<Duration>,
    /// Retry configuration
    pub retry: Option<RetryConfig>,
    /// Enable streaming response
    pub streaming: bool,
}

/// Retry configuration for function invocation
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_attempts: u32,
    /// Delay between retries
    pub delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum delay between retries
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(30),
        }
    }
}

/// Streaming chunk from function response
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Chunk data
    pub data: Value,
    /// Chunk sequence number
    pub sequence: Option<u64>,
    /// Whether this is the last chunk
    pub is_final: bool,
}

/// Local development configuration
#[derive(Debug, Clone)]
pub struct LocalConfig {
    /// Local functions server URL
    pub local_url: String,
    /// Local functions directory
    pub functions_dir: Option<String>,
    /// Development server port
    pub port: Option<u16>,
}

impl Functions {
    /// Create a new Functions instance
    pub fn new(config: Arc<SupabaseConfig>, http_client: Arc<HttpClient>) -> Result<Self> {
        debug!("Initializing Functions module");

        Ok(Self {
            http_client,
            config,
        })
    }

    /// Invoke an Edge Function
    ///
    /// # Parameters
    ///
    /// * `function_name` - Name of the function to invoke
    /// * `body` - Optional JSON body to send to the function
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use serde_json::json;
    ///
    /// # async fn example(functions: &supabase::Functions) -> supabase::Result<()> {
    /// // Simple function call
    /// let result = functions.invoke("hello", None).await?;
    ///
    /// // Function with parameters
    /// let result = functions.invoke("process-data", Some(json!({
    ///     "user_id": 123,
    ///     "action": "update_profile"
    /// }))).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn invoke(&self, function_name: &str, body: Option<Value>) -> Result<Value> {
        self.invoke_with_options(function_name, body, None).await
    }

    /// Invoke an Edge Function with custom options
    ///
    /// # Parameters
    ///
    /// * `function_name` - Name of the function to invoke
    /// * `body` - Optional JSON body to send to the function
    /// * `headers` - Optional additional headers to send
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use supabase::Client;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> supabase::Result<()> {
    /// let client = Client::new("your-project-url", "your-anon-key")?;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
    ///
    /// let result = client.functions()
    ///     .invoke_with_options("my-function", Some(json!({"data": "value"})), Some(headers))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn invoke_with_options(
        &self,
        function_name: &str,
        body: Option<Value>,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Value> {
        debug!("Invoking Edge Function: {}", function_name);

        let url = format!("{}/functions/v1/{}", self.config.url, function_name);

        let mut request = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.key))
            .header("Content-Type", "application/json");

        // Add custom headers if provided
        if let Some(custom_headers) = headers {
            for (key, value) in custom_headers {
                request = request.header(key, value);
            }
        }

        // Add body if provided
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => {
                    // Try to parse error message from Supabase
                    if let Ok(error_json) = serde_json::from_str::<Value>(&text) {
                        if let Some(message) = error_json.get("message") {
                            message.as_str().unwrap_or(&text).to_string()
                        } else {
                            text
                        }
                    } else {
                        text
                    }
                }
                Err(_) => format!("Function invocation failed with status: {}", status),
            };
            return Err(Error::functions(error_msg));
        }

        let result: Value = response.json().await?;
        info!("Edge Function {} invoked successfully", function_name);

        Ok(result)
    }

    /// Invoke an Edge Function with streaming response (native only)
    ///
    /// This method enables server-sent events or streaming responses from functions.
    /// Only available on native platforms (not WASM).
    ///
    /// # Parameters
    ///
    /// * `function_name` - Name of the function to invoke
    /// * `body` - Optional JSON body to send to the function
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use serde_json::json;
    /// use tokio_stream::StreamExt;
    ///
    /// # async fn example(functions: &supabase::Functions) -> supabase::Result<()> {
    /// let mut stream = functions.invoke_stream("streaming-function", Some(json!({
    ///     "mode": "realtime",
    ///     "duration": 60
    /// }))).await?;
    ///
    /// while let Some(chunk) = stream.next().await {
    ///     match chunk {
    ///         Ok(data) => println!("Received chunk: {}", data.data),
    ///         Err(e) => println!("Stream error: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn invoke_stream(
        &self,
        function_name: &str,
        body: Option<Value>,
    ) -> Result<impl Stream<Item = Result<StreamChunk>>> {
        debug!(
            "Starting streaming invocation of function: {}",
            function_name
        );

        let url = format!("{}/functions/v1/{}", self.config.url, function_name);

        let mut request = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.key))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache");

        // Add body if provided
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = response.text().await.unwrap_or_else(|_| {
                format!(
                    "Streaming function invocation failed with status: {}",
                    status
                )
            });
            return Err(Error::functions(error_msg));
        }

        self.process_stream(response).await
    }

    /// Get metadata for a specific function
    ///
    /// # Parameters
    ///
    /// * `function_name` - Name of the function to introspect
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example(functions: &supabase::Functions) -> supabase::Result<()> {
    /// let metadata = functions.get_function_metadata("my-function").await?;
    /// println!("Function: {}", metadata.name);
    /// println!("Status: {:?}", metadata.status);
    /// println!("Memory: {:?} MB", metadata.memory_limit);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_function_metadata(&self, function_name: &str) -> Result<FunctionMetadata> {
        debug!("Fetching metadata for function: {}", function_name);

        let url = format!(
            "{}/functions/v1/{}/metadata",
            self.config.url, function_name
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.key))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = response.text().await.unwrap_or_else(|_| {
                format!("Failed to fetch function metadata, status: {}", status)
            });
            return Err(Error::functions(error_msg));
        }

        let metadata: FunctionMetadata = response.json().await?;
        info!("Retrieved metadata for function: {}", function_name);

        Ok(metadata)
    }

    /// List all available functions with their metadata
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example(functions: &supabase::Functions) -> supabase::Result<()> {
    /// let functions_list = functions.list_functions().await?;
    /// for func in functions_list {
    ///     println!("Function: {} - Status: {:?}", func.name, func.status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_functions(&self) -> Result<Vec<FunctionMetadata>> {
        debug!("Listing all available functions");

        let url = format!("{}/functions/v1", self.config.url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.key))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| format!("Failed to list functions, status: {}", status));
            return Err(Error::functions(error_msg));
        }

        let functions: Vec<FunctionMetadata> = response.json().await?;
        info!("Retrieved {} functions", functions.len());

        Ok(functions)
    }

    /// Invoke a function with advanced options
    ///
    /// # Parameters
    ///
    /// * `function_name` - Name of the function to invoke
    /// * `body` - Optional JSON body to send to the function
    /// * `options` - Invocation options (headers, timeout, retry, etc.)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use supabase::functions::{InvokeOptions, RetryConfig};
    /// use serde_json::json;
    /// use std::{collections::HashMap, time::Duration};
    ///
    /// # async fn example(functions: &supabase::Functions) -> supabase::Result<()> {
    /// let mut headers = HashMap::new();
    /// headers.insert("X-Priority".to_string(), "high".to_string());
    ///
    /// let options = InvokeOptions {
    ///     headers: Some(headers),
    ///     timeout: Some(Duration::from_secs(30)),
    ///     retry: Some(RetryConfig::default()),
    ///     streaming: false,
    /// };
    ///
    /// let result = functions.invoke_with_advanced_options(
    ///     "critical-function",
    ///     Some(json!({"data": "important"})),
    ///     options
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn invoke_with_advanced_options(
        &self,
        function_name: &str,
        body: Option<Value>,
        options: InvokeOptions,
    ) -> Result<Value> {
        debug!("Invoking function with advanced options: {}", function_name);

        let mut attempt = 0;
        let max_attempts = options.retry.as_ref().map(|r| r.max_attempts).unwrap_or(1);

        loop {
            attempt += 1;

            match self
                .invoke_function_once(function_name, body.clone(), &options)
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) if attempt < max_attempts => {
                    warn!("Function invocation attempt {} failed: {}", attempt, e);

                    if let Some(retry_config) = &options.retry {
                        let base_delay_ms = retry_config.delay.as_millis() as u64;
                        let backoff_factor =
                            retry_config.backoff_multiplier.powi(attempt as i32 - 1);
                        let calculated_delay_ms = (base_delay_ms as f64 * backoff_factor) as u64;
                        let max_delay_ms = retry_config.max_delay.as_millis() as u64;

                        let delay_ms = std::cmp::min(calculated_delay_ms, max_delay_ms);
                        async_sleep(Duration::from_millis(delay_ms)).await;
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Test a function locally (for development)
    ///
    /// # Parameters
    ///
    /// * `function_name` - Name of the function to test
    /// * `body` - Optional JSON body to send to the function
    /// * `local_config` - Local development configuration
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use supabase::functions::LocalConfig;
    /// use serde_json::json;
    ///
    /// # async fn example(functions: &supabase::Functions) -> supabase::Result<()> {
    /// let local_config = LocalConfig {
    ///     local_url: "http://localhost:54321".to_string(),
    ///     functions_dir: Some("./functions".to_string()),
    ///     port: Some(54321),
    /// };
    ///
    /// let result = functions.test_local(
    ///     "my-function",
    ///     Some(json!({"test": true})),
    ///     local_config
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn test_local(
        &self,
        function_name: &str,
        body: Option<Value>,
        local_config: LocalConfig,
    ) -> Result<Value> {
        debug!("Testing function locally: {}", function_name);

        let url = format!("{}/functions/v1/{}", local_config.local_url, function_name);

        let mut request = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.key))
            .header("Content-Type", "application/json")
            .header("X-Local-Test", "true");

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| format!("Local function test failed with status: {}", status));
            return Err(Error::functions(error_msg));
        }

        let result: Value = response.json().await?;
        info!("Local function test completed: {}", function_name);

        Ok(result)
    }

    /// Get the base Functions URL
    pub fn functions_url(&self) -> String {
        format!("{}/functions/v1", self.config.url)
    }

    // Private helper methods

    async fn invoke_function_once(
        &self,
        function_name: &str,
        body: Option<Value>,
        options: &InvokeOptions,
    ) -> Result<Value> {
        let url = format!("{}/functions/v1/{}", self.config.url, function_name);

        let mut request = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.key))
            .header("Content-Type", "application/json");

        // Add custom headers
        if let Some(custom_headers) = &options.headers {
            for (key, value) in custom_headers {
                request = request.header(key, value);
            }
        }

        // Set timeout
        if let Some(timeout) = options.timeout {
            request = request.timeout(timeout);
        }

        // Add body if provided
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => {
                    // Enhanced error parsing
                    if let Ok(error_json) = serde_json::from_str::<Value>(&text) {
                        self.parse_function_error(&error_json)
                    } else {
                        text
                    }
                }
                Err(_) => format!("Function invocation failed with status: {}", status),
            };
            return Err(Error::functions(error_msg));
        }

        let result: Value = response.json().await?;
        Ok(result)
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn process_stream(
        &self,
        response: Response,
    ) -> Result<impl Stream<Item = Result<StreamChunk>>> {
        use tokio_stream::StreamExt;

        // Simplified streaming - read response as text and split by lines
        let text = response.text().await?;
        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();

        let stream = tokio_stream::iter(lines.into_iter().map(Ok::<String, Error>));

        Ok(
            stream.map(|line_result: Result<String>| -> Result<StreamChunk> {
                let line = line_result?;

                // Parse Server-Sent Events format
                if let Some(data_str) = line.strip_prefix("data: ") {
                    // Remove "data: " prefix
                    if data_str == "[DONE]" {
                        return Ok(StreamChunk {
                            data: Value::Null,
                            sequence: None,
                            is_final: true,
                        });
                    }

                    let data: Value = serde_json::from_str(data_str).map_err(|e| {
                        Error::functions(format!("Failed to parse stream data: {}", e))
                    })?;

                    Ok(StreamChunk {
                        data,
                        sequence: None,
                        is_final: false,
                    })
                } else if !line.is_empty() && !line.starts_with(':') {
                    // Skip non-data lines (event:, id:, etc.) and empty lines
                    Ok(StreamChunk {
                        data: Value::Null,
                        sequence: None,
                        is_final: false,
                    })
                } else {
                    Ok(StreamChunk {
                        data: Value::Null,
                        sequence: None,
                        is_final: false,
                    })
                }
            }),
        )
    }

    fn parse_function_error(&self, error_json: &Value) -> String {
        // Enhanced error parsing for different error formats
        if let Some(message) = error_json.get("error") {
            if let Some(details) = message.get("message") {
                return details.as_str().unwrap_or("Unknown error").to_string();
            }
            return message.as_str().unwrap_or("Unknown error").to_string();
        }

        if let Some(message) = error_json.get("message") {
            return message.as_str().unwrap_or("Unknown error").to_string();
        }

        if let Some(details) = error_json.get("details") {
            return details.as_str().unwrap_or("Unknown error").to_string();
        }

        "Function execution failed".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AuthConfig, DatabaseConfig, HttpConfig, StorageConfig, SupabaseConfig};

    fn create_test_functions() -> Functions {
        let config = Arc::new(SupabaseConfig {
            url: "http://localhost:54321".to_string(),
            key: "test-key".to_string(),
            service_role_key: None,
            http_config: HttpConfig::default(),
            auth_config: AuthConfig::default(),
            database_config: DatabaseConfig::default(),
            storage_config: StorageConfig::default(),
        });

        let http_client = Arc::new(HttpClient::new());
        Functions::new(config, http_client).unwrap()
    }

    #[test]
    fn test_functions_creation() {
        let functions = create_test_functions();
        assert_eq!(
            functions.functions_url(),
            "http://localhost:54321/functions/v1"
        );
    }

    #[test]
    fn test_functions_url_generation() {
        let functions = create_test_functions();
        assert_eq!(
            functions.functions_url(),
            "http://localhost:54321/functions/v1"
        );
    }
}
