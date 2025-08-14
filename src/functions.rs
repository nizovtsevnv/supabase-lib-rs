//! Edge Functions module for Supabase
//!
//! This module provides functionality to invoke Supabase Edge Functions.
//! Edge Functions are server-side TypeScript functions that run on the edge,
//! close to your users for reduced latency.

use crate::{
    error::{Error, Result},
    types::SupabaseConfig,
};
use reqwest::Client as HttpClient;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, info};

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
/// Function with custom headers:
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
#[derive(Debug, Clone)]
pub struct Functions {
    http_client: Arc<HttpClient>,
    config: Arc<SupabaseConfig>,
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
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// # async fn example(functions: &supabase::Functions) -> supabase::Result<()> {
    /// let mut headers = HashMap::new();
    /// headers.insert("X-API-Version".to_string(), "v1".to_string());
    /// headers.insert("X-Custom-Auth".to_string(), "bearer token".to_string());
    ///
    /// let result = functions.invoke_with_options(
    ///     "secure-function",
    ///     Some(json!({"sensitive": "data"})),
    ///     Some(headers)
    /// ).await?;
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

    /// Get the base Functions URL
    pub fn functions_url(&self) -> String {
        format!("{}/functions/v1", self.config.url)
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
