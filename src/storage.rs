//! Storage module for Supabase file operations

use crate::{
    error::{Error, Result},
    types::{SupabaseConfig, Timestamp},
};
use bytes::Bytes;

#[cfg(target_arch = "wasm32")]
use reqwest::Client as HttpClient;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::{multipart, Client as HttpClient};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use tracing::{debug, info};
use url::Url;

/// Storage client for file operations
#[derive(Debug, Clone)]
pub struct Storage {
    http_client: Arc<HttpClient>,
    config: Arc<SupabaseConfig>,
}

/// Storage bucket information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub id: String,
    pub name: String,
    pub owner: Option<String>,
    pub public: bool,
    pub file_size_limit: Option<u64>,
    pub allowed_mime_types: Option<Vec<String>>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// File object information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileObject {
    pub name: String,
    pub id: Option<String>,
    pub updated_at: Option<Timestamp>,
    pub created_at: Option<Timestamp>,
    pub last_accessed_at: Option<Timestamp>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Upload response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Id")]
    pub id: Option<String>,
}

/// File options for upload
#[derive(Debug, Clone, Default)]
pub struct FileOptions {
    pub cache_control: Option<String>,
    pub content_type: Option<String>,
    pub upsert: bool,
}

/// Transform options for image processing
#[derive(Debug, Clone)]
pub struct TransformOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resize: Option<ResizeMode>,
    pub format: Option<ImageFormat>,
    pub quality: Option<u8>,
}

/// Resize mode for image transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResizeMode {
    Cover,
    Contain,
    Fill,
}

/// Image format for transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Webp,
    Jpeg,
    Png,
    Avif,
}

impl Storage {
    /// Create a new Storage instance
    pub fn new(config: Arc<SupabaseConfig>, http_client: Arc<HttpClient>) -> Result<Self> {
        debug!("Initializing Storage module");

        Ok(Self {
            http_client,
            config,
        })
    }

    /// Get the appropriate authorization key for admin operations
    fn get_admin_key(&self) -> &str {
        self.config
            .service_role_key
            .as_ref()
            .unwrap_or(&self.config.key)
    }

    /// List all storage buckets
    pub async fn list_buckets(&self) -> Result<Vec<Bucket>> {
        debug!("Listing all storage buckets");

        let url = format!("{}/storage/v1/bucket", self.config.url);
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("List buckets failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        let buckets: Vec<Bucket> = response.json().await?;
        info!("Listed {} buckets successfully", buckets.len());

        Ok(buckets)
    }

    /// Get bucket information
    pub async fn get_bucket(&self, bucket_id: &str) -> Result<Bucket> {
        debug!("Getting bucket info for: {}", bucket_id);

        let url = format!("{}/storage/v1/bucket/{}", self.config.url, bucket_id);
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Get bucket failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        let bucket: Bucket = response.json().await?;
        info!("Retrieved bucket info for: {}", bucket_id);

        Ok(bucket)
    }

    /// Create a new storage bucket
    pub async fn create_bucket(&self, id: &str, name: &str, public: bool) -> Result<Bucket> {
        debug!("Creating bucket: {} ({})", name, id);

        let payload = serde_json::json!({
            "id": id,
            "name": name,
            "public": public
        });

        let url = format!("{}/storage/v1/bucket", self.config.url);
        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.get_admin_key()))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Create bucket failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        let bucket: Bucket = response.json().await?;
        info!("Created bucket successfully: {}", id);

        Ok(bucket)
    }

    /// Update bucket settings
    pub async fn update_bucket(&self, id: &str, public: Option<bool>) -> Result<()> {
        debug!("Updating bucket: {}", id);

        let mut payload = serde_json::Map::new();
        if let Some(public) = public {
            payload.insert("public".to_string(), serde_json::Value::Bool(public));
        }

        let url = format!("{}/storage/v1/bucket/{}", self.config.url, id);
        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.get_admin_key()))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Update bucket failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        info!("Updated bucket successfully: {}", id);
        Ok(())
    }

    /// Delete a storage bucket
    pub async fn delete_bucket(&self, id: &str) -> Result<()> {
        debug!("Deleting bucket: {}", id);

        let url = format!("{}/storage/v1/bucket/{}", self.config.url, id);
        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.get_admin_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Delete bucket failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        info!("Deleted bucket successfully: {}", id);
        Ok(())
    }

    /// List files in a bucket
    pub async fn list(&self, bucket_id: &str, path: Option<&str>) -> Result<Vec<FileObject>> {
        debug!("Listing files in bucket: {}", bucket_id);

        let url = format!("{}/storage/v1/object/list/{}", self.config.url, bucket_id);

        let payload = serde_json::json!({
            "prefix": path.unwrap_or("")
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("List files failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        let files: Vec<FileObject> = response.json().await?;
        info!("Listed {} files in bucket: {}", files.len(), bucket_id);

        Ok(files)
    }

    /// Upload a file from bytes
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn upload(
        &self,
        bucket_id: &str,
        path: &str,
        file_body: Bytes,
        options: Option<FileOptions>,
    ) -> Result<UploadResponse> {
        debug!("Uploading file to bucket: {} at path: {}", bucket_id, path);

        let options = options.unwrap_or_default();

        let url = format!(
            "{}/storage/v1/object/{}/{}",
            self.config.url, bucket_id, path
        );

        let mut form = multipart::Form::new().part(
            "file",
            multipart::Part::bytes(file_body.to_vec()).file_name(path.to_string()),
        );

        if let Some(content_type) = options.content_type {
            form = form.part("contentType", multipart::Part::text(content_type));
        }

        if let Some(cache_control) = options.cache_control {
            form = form.part("cacheControl", multipart::Part::text(cache_control));
        }

        let mut request = self.http_client.post(&url).multipart(form);

        if options.upsert {
            request = request.header("x-upsert", "true");
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Upload failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        let upload_response: UploadResponse = response.json().await?;
        info!("Uploaded file successfully: {}", path);
        Ok(upload_response)
    }

    /// Upload a file from bytes (WASM version)
    ///
    /// Note: WASM version uses simpler body upload due to multipart limitations
    #[cfg(target_arch = "wasm32")]
    pub async fn upload(
        &self,
        bucket_id: &str,
        path: &str,
        file_body: Bytes,
        options: Option<FileOptions>,
    ) -> Result<UploadResponse> {
        debug!(
            "Uploading file to bucket: {} at path: {} (WASM)",
            bucket_id, path
        );

        let options = options.unwrap_or_default();

        let url = format!(
            "{}/storage/v1/object/{}/{}",
            self.config.url, bucket_id, path
        );

        let mut request = self.http_client.post(&url).body(file_body);

        if let Some(content_type) = options.content_type {
            request = request.header("Content-Type", content_type);
        }

        if let Some(cache_control) = options.cache_control {
            request = request.header("Cache-Control", cache_control);
        }

        if options.upsert {
            request = request.header("x-upsert", "true");
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Upload failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        let upload_response: UploadResponse = response.json().await?;
        info!("Uploaded file successfully: {}", path);

        Ok(upload_response)
    }

    /// Upload a file from local filesystem (Native only, requires tokio)
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    pub async fn upload_file<P: AsRef<std::path::Path>>(
        &self,
        bucket_id: &str,
        path: &str,
        file_path: P,
        options: Option<FileOptions>,
    ) -> Result<UploadResponse> {
        debug!("Uploading file from path: {:?}", file_path.as_ref());

        let file_bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| Error::storage(format!("Failed to read file: {}", e)))?;

        self.upload(bucket_id, path, Bytes::from(file_bytes), options)
            .await
    }

    /// Download a file
    pub async fn download(&self, bucket_id: &str, path: &str) -> Result<Bytes> {
        debug!(
            "Downloading file from bucket: {} at path: {}",
            bucket_id, path
        );

        let url = format!(
            "{}/storage/v1/object/{}/{}",
            self.config.url, bucket_id, path
        );

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            let error_msg = format!("Download failed with status: {}", response.status());
            return Err(Error::storage(error_msg));
        }

        let bytes = response.bytes().await?;
        info!("Downloaded file successfully: {}", path);

        Ok(bytes)
    }

    /// Delete a file
    pub async fn remove(&self, bucket_id: &str, paths: &[&str]) -> Result<()> {
        debug!("Deleting files from bucket: {}", bucket_id);

        let url = format!("{}/storage/v1/object/{}", self.config.url, bucket_id);

        let payload = serde_json::json!({
            "prefixes": paths
        });

        let response = self.http_client.delete(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Delete failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        info!("Deleted {} files successfully", paths.len());
        Ok(())
    }

    /// Move a file
    pub async fn r#move(&self, bucket_id: &str, from_path: &str, to_path: &str) -> Result<()> {
        debug!("Moving file from {} to {}", from_path, to_path);

        let url = format!("{}/storage/v1/object/move", self.config.url);

        let payload = serde_json::json!({
            "bucketId": bucket_id,
            "sourceKey": from_path,
            "destinationKey": to_path
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Move failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        info!("Moved file successfully from {} to {}", from_path, to_path);
        Ok(())
    }

    /// Copy a file
    pub async fn copy(&self, bucket_id: &str, from_path: &str, to_path: &str) -> Result<()> {
        debug!("Copying file from {} to {}", from_path, to_path);

        let url = format!("{}/storage/v1/object/copy", self.config.url);

        let payload = serde_json::json!({
            "bucketId": bucket_id,
            "sourceKey": from_path,
            "destinationKey": to_path
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Copy failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        info!("Copied file successfully from {} to {}", from_path, to_path);
        Ok(())
    }

    /// Get public URL for a file
    pub fn get_public_url(&self, bucket_id: &str, path: &str) -> String {
        format!(
            "{}/storage/v1/object/public/{}/{}",
            self.config.url, bucket_id, path
        )
    }

    /// Create a signed URL for private file access
    pub async fn create_signed_url(
        &self,
        bucket_id: &str,
        path: &str,
        expires_in_seconds: u64,
    ) -> Result<String> {
        debug!("Creating signed URL for file: {}", path);

        let url = format!("{}/storage/v1/object/sign/{}", self.config.url, bucket_id);

        let payload = serde_json::json!({
            "expiresIn": expires_in_seconds,
            "paths": [path]
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Create signed URL failed with status: {}", status),
            };
            return Err(Error::storage(error_msg));
        }

        let result: serde_json::Value = response.json().await?;
        let signed_urls = result["signedUrls"].as_array().ok_or_else(|| {
            Error::storage("Invalid signed URL response - missing signedUrls array")
        })?;

        let first = signed_urls.first().ok_or_else(|| {
            Error::storage("Invalid signed URL response - empty signedUrls array")
        })?;

        let first_url = first["signedUrl"].as_str().ok_or_else(|| {
            Error::storage("Invalid signed URL response - missing signedUrl field")
        })?;

        info!("Created signed URL successfully for: {}", path);
        Ok(first_url.to_string())
    }

    /// Get transformed image URL
    pub fn get_public_url_transformed(
        &self,
        bucket_id: &str,
        path: &str,
        options: TransformOptions,
    ) -> Result<String> {
        let mut url = Url::parse(&self.get_public_url(bucket_id, path))?;

        if let Some(width) = options.width {
            url.query_pairs_mut()
                .append_pair("width", &width.to_string());
        }

        if let Some(height) = options.height {
            url.query_pairs_mut()
                .append_pair("height", &height.to_string());
        }

        if let Some(resize) = options.resize {
            let resize_str = match resize {
                ResizeMode::Cover => "cover",
                ResizeMode::Contain => "contain",
                ResizeMode::Fill => "fill",
            };
            url.query_pairs_mut().append_pair("resize", resize_str);
        }

        if let Some(format) = options.format {
            let format_str = match format {
                ImageFormat::Webp => "webp",
                ImageFormat::Jpeg => "jpeg",
                ImageFormat::Png => "png",
                ImageFormat::Avif => "avif",
            };
            url.query_pairs_mut().append_pair("format", format_str);
        }

        if let Some(quality) = options.quality {
            url.query_pairs_mut()
                .append_pair("quality", &quality.to_string());
        }

        Ok(url.to_string())
    }
}
