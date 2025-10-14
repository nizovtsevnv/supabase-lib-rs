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

#[cfg(target_arch = "wasm32")]
use tracing::{debug, info};
#[cfg(not(target_arch = "wasm32"))]
use tracing::{debug, info, warn};
use url::Url;

// Resumable uploads support
#[cfg(target_arch = "wasm32")]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::{sleep, Duration};

// Helper for async sleep across platforms
#[cfg(not(target_arch = "wasm32"))]
async fn async_sleep(duration: Duration) {
    sleep(duration).await;
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
async fn async_sleep(duration: Duration) {
    use gloo_timers::future::sleep as gloo_sleep;
    gloo_sleep(duration).await;
}

#[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
#[allow(dead_code)]
async fn async_sleep(_duration: Duration) {
    // No-op for wasm32 without wasm feature (resumable uploads not fully supported)
}

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

/// Resumable upload session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSession {
    pub upload_id: String,
    pub part_size: u64,
    pub total_size: u64,
    pub uploaded_parts: Vec<UploadedPart>,
    pub bucket_id: String,
    pub object_path: String,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
}

/// Information about an uploaded part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedPart {
    pub part_number: u32,
    pub etag: String,
    pub size: u64,
}

/// Configuration for resumable uploads
#[derive(Debug, Clone)]
pub struct ResumableUploadConfig {
    /// Size of each chunk (default: 5MB)
    pub chunk_size: u64,
    /// Maximum retry attempts (default: 3)
    pub max_retries: u32,
    /// Retry delay in milliseconds (default: 1000)
    pub retry_delay: u64,
    /// Whether to verify uploaded chunks with checksums (default: true)
    pub verify_checksums: bool,
}

impl Default for ResumableUploadConfig {
    fn default() -> Self {
        Self {
            chunk_size: 5 * 1024 * 1024, // 5MB
            max_retries: 3,
            retry_delay: 1000,
            verify_checksums: true,
        }
    }
}

/// Progress callback for resumable uploads
pub type UploadProgressCallback = Arc<dyn Fn(u64, u64) + Send + Sync>;

/// Advanced metadata for files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub tags: Option<HashMap<String, String>>,
    pub custom_metadata: Option<HashMap<String, serde_json::Value>>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub searchable_content: Option<String>,
}

/// Search options for file metadata
#[derive(Debug, Clone, Default, Serialize)]
pub struct SearchOptions {
    pub tags: Option<HashMap<String, String>>,
    pub category: Option<String>,
    pub content_search: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Storage event types for real-time notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageEvent {
    #[serde(rename = "file_uploaded")]
    FileUploaded,
    #[serde(rename = "file_deleted")]
    FileDeleted,
    #[serde(rename = "file_updated")]
    FileUpdated,
    #[serde(rename = "bucket_created")]
    BucketCreated,
    #[serde(rename = "bucket_deleted")]
    BucketDeleted,
}

/// Storage event notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEventMessage {
    pub event: StorageEvent,
    pub bucket_id: String,
    pub object_path: Option<String>,
    pub object_metadata: Option<FileObject>,
    pub timestamp: Timestamp,
    pub user_id: Option<String>,
}

/// Callback for storage events
pub type StorageEventCallback = Arc<dyn Fn(StorageEventMessage) + Send + Sync>;

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
            let error_msg = format!("Delete files failed with status: {}", response.status());
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

    /// Get signed URL for private file access
    pub async fn create_signed_url(
        &self,
        bucket_id: &str,
        path: &str,
        expires_in: u32,
        transform: Option<TransformOptions>,
    ) -> Result<String> {
        debug!(
            "Creating signed URL for bucket: {} path: {} expires_in: {}",
            bucket_id, path, expires_in
        );

        let url = format!(
            "{}/storage/v1/object/sign/{}/{}",
            self.config.url, bucket_id, path
        );

        let mut payload = serde_json::json!({
            "expiresIn": expires_in
        });

        if let Some(transform_opts) = transform {
            let mut transform_params = serde_json::Map::new();

            if let Some(width) = transform_opts.width {
                transform_params.insert("width".to_string(), serde_json::Value::from(width));
            }
            if let Some(height) = transform_opts.height {
                transform_params.insert("height".to_string(), serde_json::Value::from(height));
            }
            if let Some(resize) = transform_opts.resize {
                transform_params.insert("resize".to_string(), serde_json::to_value(resize)?);
            }
            if let Some(format) = transform_opts.format {
                transform_params.insert("format".to_string(), serde_json::to_value(format)?);
            }
            if let Some(quality) = transform_opts.quality {
                transform_params.insert("quality".to_string(), serde_json::Value::from(quality));
            }

            payload["transform"] = serde_json::Value::Object(transform_params);
        }

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Create signed URL failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        let response_data: serde_json::Value = response.json().await?;
        let signed_url = response_data["signedURL"]
            .as_str()
            .ok_or_else(|| Error::storage("Invalid signed URL response"))?;

        info!("Created signed URL successfully");
        Ok(signed_url.to_string())
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

    /// Start a resumable upload session for large files
    ///
    /// # Examples
    /// ```rust,no_run
    /// use supabase::storage::{ResumableUploadConfig, FileOptions};
    ///
    /// # async fn example(storage: &supabase::storage::Storage) -> supabase::Result<()> {
    /// let config = ResumableUploadConfig::default();
    /// let file_opts = FileOptions {
    ///     content_type: Some("video/mp4".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// let session = storage.start_resumable_upload(
    ///     "videos",
    ///     "my-large-video.mp4",
    ///     1024 * 1024 * 100, // 100MB
    ///     Some(config),
    ///     Some(file_opts)
    /// ).await?;
    ///
    /// println!("Started upload session: {}", session.upload_id);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn start_resumable_upload(
        &self,
        bucket_id: &str,
        path: &str,
        total_size: u64,
        config: Option<ResumableUploadConfig>,
        options: Option<FileOptions>,
    ) -> Result<UploadSession> {
        let config = config.unwrap_or_default();
        let options = options.unwrap_or_default();

        debug!(
            "Starting resumable upload for bucket: {} path: {} size: {}",
            bucket_id, path, total_size
        );

        let url = format!(
            "{}/storage/v1/object/{}/{}/resumable",
            self.config.url, bucket_id, path
        );

        let payload = serde_json::json!({
            "totalSize": total_size,
            "chunkSize": config.chunk_size,
            "contentType": options.content_type,
            "cacheControl": options.cache_control,
            "upsert": options.upsert
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Start resumable upload failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        let session: UploadSession = response.json().await?;
        info!("Started resumable upload session: {}", session.upload_id);

        Ok(session)
    }

    /// Upload a chunk for resumable upload
    ///
    /// # Examples
    /// ```rust,no_run
    /// use bytes::Bytes;
    ///
    /// # async fn example(storage: &supabase::storage::Storage, session: &supabase::storage::UploadSession) -> supabase::Result<()> {
    /// let chunk_data = Bytes::from(vec![0u8; 1024 * 1024]); // 1MB chunk
    ///
    /// let part = storage.upload_chunk(
    ///     session,
    ///     1, // part number
    ///     chunk_data
    /// ).await?;
    ///
    /// println!("Uploaded part: {} etag: {}", part.part_number, part.etag);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn upload_chunk(
        &self,
        session: &UploadSession,
        part_number: u32,
        chunk_data: Bytes,
    ) -> Result<UploadedPart> {
        debug!(
            "Uploading chunk {} for session: {} size: {}",
            part_number,
            session.upload_id,
            chunk_data.len()
        );

        let url = format!(
            "{}/storage/v1/object/{}/{}/resumable/{}",
            self.config.url, session.bucket_id, session.object_path, session.upload_id
        );

        let chunk_size = chunk_data.len() as u64;

        let response = self
            .http_client
            .put(&url)
            .header("Content-Type", "application/octet-stream")
            .header("X-Part-Number", part_number.to_string())
            .body(chunk_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!("Upload chunk failed with status: {}", response.status());
            return Err(Error::storage(error_msg));
        }

        let etag = response
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_string();

        let part = UploadedPart {
            part_number,
            etag,
            size: chunk_size,
        };

        info!("Uploaded chunk {} successfully", part_number);
        Ok(part)
    }

    /// Complete a resumable upload after all chunks are uploaded
    ///
    /// # Examples
    /// ```rust,no_run
    /// # async fn example(storage: &supabase::storage::Storage, mut session: supabase::storage::UploadSession) -> supabase::Result<()> {
    /// // ... upload all chunks and collect parts ...
    ///
    /// let response = storage.complete_resumable_upload(&session).await?;
    /// println!("Upload completed: {}", response.key);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn complete_resumable_upload(
        &self,
        session: &UploadSession,
    ) -> Result<UploadResponse> {
        debug!(
            "Completing resumable upload for session: {}",
            session.upload_id
        );

        let url = format!(
            "{}/storage/v1/object/{}/{}/resumable/{}/complete",
            self.config.url, session.bucket_id, session.object_path, session.upload_id
        );

        let payload = serde_json::json!({
            "parts": session.uploaded_parts
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Complete resumable upload failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        let upload_response: UploadResponse = response.json().await?;
        info!("Completed resumable upload: {}", upload_response.key);

        Ok(upload_response)
    }

    /// Upload a large file with automatic chunking and resume capability
    ///
    /// This is a high-level method that handles the entire resumable upload process.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use supabase::storage::{ResumableUploadConfig, FileOptions};
    /// use std::sync::Arc;
    ///
    /// # async fn example(storage: &supabase::storage::Storage) -> supabase::Result<()> {
    /// let config = ResumableUploadConfig::default();
    /// let file_opts = FileOptions {
    ///     content_type: Some("video/mp4".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// let progress_callback = Arc::new(|uploaded: u64, total: u64| {
    ///     println!("Progress: {:.1}%", (uploaded as f64 / total as f64) * 100.0);
    /// });
    ///
    /// let response = storage.upload_large_file(
    ///     "videos",
    ///     "my-large-video.mp4",
    ///     "/path/to/large-video.mp4",
    ///     Some(config),
    ///     Some(file_opts),
    ///     Some(progress_callback)
    /// ).await?;
    ///
    /// println!("Upload completed: {}", response.key);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    pub async fn upload_large_file<P: AsRef<std::path::Path>>(
        &self,
        bucket_id: &str,
        path: &str,
        file_path: P,
        config: Option<ResumableUploadConfig>,
        options: Option<FileOptions>,
        progress_callback: Option<UploadProgressCallback>,
    ) -> Result<UploadResponse> {
        let config = config.unwrap_or_default();

        debug!("Starting large file upload from: {:?}", file_path.as_ref());

        // Get file size
        let metadata = tokio::fs::metadata(&file_path)
            .await
            .map_err(|e| Error::storage(format!("Failed to get file metadata: {}", e)))?;

        let total_size = metadata.len();

        if total_size <= config.chunk_size {
            // Use regular upload for small files
            return self.upload_file(bucket_id, path, file_path, options).await;
        }

        // Start resumable upload session
        let mut session = self
            .start_resumable_upload(bucket_id, path, total_size, Some(config.clone()), options)
            .await?;

        // Open file for reading
        let mut file = tokio::fs::File::open(&file_path)
            .await
            .map_err(|e| Error::storage(format!("Failed to open file: {}", e)))?;

        let mut uploaded_size = 0u64;
        let mut part_number = 1u32;

        // Upload chunks
        loop {
            let remaining_size = total_size - uploaded_size;
            if remaining_size == 0 {
                break;
            }

            let chunk_size = std::cmp::min(config.chunk_size, remaining_size);
            let mut buffer = vec![0u8; chunk_size as usize];

            // Read chunk from file
            use tokio::io::AsyncReadExt;
            let bytes_read = file
                .read_exact(&mut buffer)
                .await
                .map_err(|e| Error::storage(format!("Failed to read file chunk: {}", e)))?;

            if bytes_read == 0 {
                break;
            }

            buffer.truncate(bytes_read);
            let chunk_data = Bytes::from(buffer);

            // Upload chunk with retries
            let mut attempts = 0;
            let part = loop {
                attempts += 1;

                match self
                    .upload_chunk(&session, part_number, chunk_data.clone())
                    .await
                {
                    Ok(part) => break part,
                    Err(e) if attempts < config.max_retries => {
                        warn!(
                            "Upload chunk {} failed (attempt {}), retrying: {}",
                            part_number, attempts, e
                        );
                        async_sleep(Duration::from_millis(config.retry_delay)).await;
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            };

            session.uploaded_parts.push(part);
            uploaded_size += chunk_size;
            part_number += 1;

            // Call progress callback
            if let Some(callback) = &progress_callback {
                callback(uploaded_size, total_size);
            }

            debug!(
                "Uploaded chunk {}, progress: {}/{}",
                part_number - 1,
                uploaded_size,
                total_size
            );
        }

        // Complete upload
        let response = self.complete_resumable_upload(&session).await?;

        info!("Large file upload completed: {}", response.key);
        Ok(response)
    }

    /// Get resumable upload session status
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_upload_session(&self, upload_id: &str) -> Result<UploadSession> {
        debug!("Getting upload session status: {}", upload_id);

        let url = format!("{}/storage/v1/resumable/{}", self.config.url, upload_id);

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Get upload session failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        let session: UploadSession = response.json().await?;
        Ok(session)
    }

    /// Cancel a resumable upload session
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn cancel_upload_session(&self, upload_id: &str) -> Result<()> {
        debug!("Cancelling upload session: {}", upload_id);

        let url = format!("{}/storage/v1/resumable/{}", self.config.url, upload_id);

        let response = self.http_client.delete(&url).send().await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Cancel upload session failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        info!("Cancelled upload session: {}", upload_id);
        Ok(())
    }

    /// Update file metadata with tags and custom metadata
    ///
    /// # Examples
    /// ```rust,no_run
    /// use std::collections::HashMap;
    /// use supabase::storage::FileMetadata;
    ///
    /// # async fn example(storage: &supabase::storage::Storage) -> supabase::Result<()> {
    /// let mut tags = HashMap::new();
    /// tags.insert("category".to_string(), "documents".to_string());
    /// tags.insert("project".to_string(), "web-app".to_string());
    ///
    /// let mut custom_data = HashMap::new();
    /// custom_data.insert("author".to_string(), serde_json::Value::String("john_doe".to_string()));
    /// custom_data.insert("version".to_string(), serde_json::Value::Number(serde_json::Number::from(1)));
    ///
    /// let metadata = FileMetadata {
    ///     tags: Some(tags),
    ///     custom_metadata: Some(custom_data),
    ///     description: Some("Project documentation".to_string()),
    ///     category: Some("documents".to_string()),
    ///     searchable_content: Some("documentation project guide".to_string()),
    /// };
    ///
    /// storage.update_file_metadata("documents", "guide.pdf", &metadata).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_file_metadata(
        &self,
        bucket_id: &str,
        path: &str,
        metadata: &FileMetadata,
    ) -> Result<()> {
        debug!(
            "Updating file metadata for bucket: {} path: {}",
            bucket_id, path
        );

        let url = format!(
            "{}/storage/v1/object/{}/{}/metadata",
            self.config.url, bucket_id, path
        );

        let response = self.http_client.put(&url).json(metadata).send().await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Update file metadata failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        info!("Updated file metadata successfully");
        Ok(())
    }

    /// Search files by metadata
    ///
    /// # Examples
    /// ```rust,no_run
    /// use std::collections::HashMap;
    /// use supabase::storage::SearchOptions;
    ///
    /// # async fn example(storage: &supabase::storage::Storage) -> supabase::Result<()> {
    /// let mut tag_filter = HashMap::new();
    /// tag_filter.insert("category".to_string(), "documents".to_string());
    ///
    /// let search_options = SearchOptions {
    ///     tags: Some(tag_filter),
    ///     category: Some("documents".to_string()),
    ///     content_search: Some("project guide".to_string()),
    ///     limit: Some(20),
    ///     offset: Some(0),
    /// };
    ///
    /// let files = storage.search_files("documents", &search_options).await?;
    /// println!("Found {} files", files.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_files(
        &self,
        bucket_id: &str,
        search_options: &SearchOptions,
    ) -> Result<Vec<FileObject>> {
        debug!("Searching files in bucket: {}", bucket_id);

        let url = format!("{}/storage/v1/object/{}/search", self.config.url, bucket_id);

        let response = self
            .http_client
            .post(&url)
            .json(search_options)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!("Search files failed with status: {}", response.status());
            return Err(Error::storage(error_msg));
        }

        let files: Vec<FileObject> = response.json().await?;
        info!("Found {} files matching search criteria", files.len());

        Ok(files)
    }

    // ========================
    // STORAGE POLICIES HELPERS
    // ========================

    /// Create a storage policy for Row Level Security (RLS)
    ///
    /// # Examples
    /// ```rust,no_run
    /// use supabase::storage::{StoragePolicy, PolicyOperation};
    ///
    /// # async fn example(storage: &supabase::storage::Storage) -> supabase::Result<()> {
    /// let policy = StoragePolicy {
    ///     name: "user_files_policy".to_string(),
    ///     bucket_id: "user-files".to_string(),
    ///     operation: PolicyOperation::Select,
    ///     definition: "auth.uid()::text = (storage.foldername(name))[1]".to_string(),
    ///     check: None,
    /// };
    ///
    /// storage.create_policy(&policy).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_policy(&self, policy: &StoragePolicy) -> Result<()> {
        debug!("Creating storage policy: {}", policy.name);

        let url = format!("{}/rest/v1/rpc/create_storage_policy", self.config.url);

        let payload = serde_json::json!({
            "policy_name": policy.name,
            "bucket_name": policy.bucket_id,
            "operation": policy.operation,
            "definition": policy.definition,
            "check_expression": policy.check
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.get_admin_key()))
            .header("apikey", self.get_admin_key())
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Create storage policy failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        info!("Created storage policy: {}", policy.name);
        Ok(())
    }

    /// Update an existing storage policy
    ///
    /// # Examples
    /// ```rust,no_run
    /// use supabase::storage::{StoragePolicy, PolicyOperation};
    ///
    /// # async fn example(storage: &supabase::storage::Storage) -> supabase::Result<()> {
    /// let updated_policy = StoragePolicy {
    ///     name: "user_files_policy".to_string(),
    ///     bucket_id: "user-files".to_string(),
    ///     operation: PolicyOperation::All,
    ///     definition: "auth.uid()::text = (storage.foldername(name))[1] OR auth.role() = 'admin'".to_string(),
    ///     check: Some("auth.uid() IS NOT NULL".to_string()),
    /// };
    ///
    /// storage.update_policy(&updated_policy).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_policy(&self, policy: &StoragePolicy) -> Result<()> {
        debug!("Updating storage policy: {}", policy.name);

        let url = format!("{}/rest/v1/rpc/update_storage_policy", self.config.url);

        let payload = serde_json::json!({
            "policy_name": policy.name,
            "bucket_name": policy.bucket_id,
            "operation": policy.operation,
            "definition": policy.definition,
            "check_expression": policy.check
        });

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.get_admin_key()))
            .header("apikey", self.get_admin_key())
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Update storage policy failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        info!("Updated storage policy: {}", policy.name);
        Ok(())
    }

    /// Delete a storage policy
    ///
    /// # Examples
    /// ```rust,no_run
    /// # async fn example(storage: &supabase::storage::Storage) -> supabase::Result<()> {
    /// storage.delete_policy("user-files", "user_files_policy").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_policy(&self, bucket_id: &str, policy_name: &str) -> Result<()> {
        debug!(
            "Deleting storage policy: {} from bucket: {}",
            policy_name, bucket_id
        );

        let url = format!("{}/rest/v1/rpc/delete_storage_policy", self.config.url);

        let payload = serde_json::json!({
            "policy_name": policy_name,
            "bucket_name": bucket_id
        });

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.get_admin_key()))
            .header("apikey", self.get_admin_key())
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Delete storage policy failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        info!("Deleted storage policy: {}", policy_name);
        Ok(())
    }

    /// List all storage policies for a bucket
    ///
    /// # Examples
    /// ```rust,no_run
    /// # async fn example(storage: &supabase::storage::Storage) -> supabase::Result<()> {
    /// let policies = storage.list_policies("user-files").await?;
    /// println!("Found {} policies", policies.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_policies(&self, bucket_id: &str) -> Result<Vec<StoragePolicy>> {
        debug!("Listing storage policies for bucket: {}", bucket_id);

        let url = format!("{}/rest/v1/rpc/list_storage_policies", self.config.url);

        let payload = serde_json::json!({
            "bucket_name": bucket_id
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.get_admin_key()))
            .header("apikey", self.get_admin_key())
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "List storage policies failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        let policies: Vec<StoragePolicy> = response.json().await?;
        info!(
            "Listed {} storage policies for bucket: {}",
            policies.len(),
            bucket_id
        );

        Ok(policies)
    }

    /// Test if a user can access a file based on current policies
    ///
    /// # Examples
    /// ```rust,no_run
    /// use supabase::storage::PolicyOperation;
    ///
    /// # async fn example(storage: &supabase::storage::Storage) -> supabase::Result<()> {
    /// let can_access = storage.test_policy_access(
    ///     "user-files",
    ///     "user123/document.pdf",
    ///     PolicyOperation::Select,
    ///     "user123"
    /// ).await?;
    ///
    /// if can_access {
    ///     println!("User can access the file");
    /// } else {
    ///     println!("Access denied");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn test_policy_access(
        &self,
        bucket_id: &str,
        object_path: &str,
        operation: PolicyOperation,
        user_id: &str,
    ) -> Result<bool> {
        debug!(
            "Testing policy access for user: {} on object: {}",
            user_id, object_path
        );

        let url = format!("{}/rest/v1/rpc/test_storage_policy_access", self.config.url);

        let payload = serde_json::json!({
            "bucket_name": bucket_id,
            "object_name": object_path,
            "operation": operation,
            "user_id": user_id
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.get_admin_key()))
            .header("apikey", self.get_admin_key())
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Test policy access failed with status: {}",
                response.status()
            );
            return Err(Error::storage(error_msg));
        }

        let result: serde_json::Value = response.json().await?;
        let can_access = result["can_access"].as_bool().unwrap_or(false);

        info!("Policy access test result: {}", can_access);
        Ok(can_access)
    }

    /// Generate a policy template for common use cases
    ///
    /// # Examples
    /// ```rust,no_run
    /// use supabase::storage::PolicyTemplate;
    ///
    /// # async fn example(storage: &supabase::storage::Storage) -> supabase::Result<()> {
    /// let policy = storage.generate_policy_template(
    ///     "user-files",
    ///     "user_files_access",
    ///     PolicyTemplate::UserFolderAccess
    /// );
    ///
    /// println!("Generated policy: {:?}", policy);
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_policy_template(
        &self,
        bucket_id: &str,
        policy_name: &str,
        template: PolicyTemplate,
    ) -> StoragePolicy {
        let (operation, definition, check) = match template {
            PolicyTemplate::PublicRead => (PolicyOperation::Select, "true".to_string(), None),
            PolicyTemplate::AuthenticatedRead => (
                PolicyOperation::Select,
                "auth.uid() IS NOT NULL".to_string(),
                None,
            ),
            PolicyTemplate::UserFolderAccess => (
                PolicyOperation::All,
                "auth.uid()::text = (storage.foldername(name))[1]".to_string(),
                Some("auth.uid() IS NOT NULL".to_string()),
            ),
            PolicyTemplate::AdminFullAccess => (
                PolicyOperation::All,
                "auth.role() = 'admin'".to_string(),
                None,
            ),
            PolicyTemplate::ReadOnlyForRole(role) => (
                PolicyOperation::Select,
                format!("auth.role() = '{}'", role),
                None,
            ),
        };

        StoragePolicy {
            name: policy_name.to_string(),
            bucket_id: bucket_id.to_string(),
            operation,
            definition,
            check,
        }
    }
}

/// Storage policy for Row Level Security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePolicy {
    pub name: String,
    pub bucket_id: String,
    pub operation: PolicyOperation,
    pub definition: String,
    pub check: Option<String>,
}

/// Policy operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PolicyOperation {
    Select,
    Insert,
    Update,
    Delete,
    All,
}

/// Pre-defined policy templates for common use cases
#[derive(Debug, Clone)]
pub enum PolicyTemplate {
    /// Allow public read access to all files
    PublicRead,
    /// Allow read access only to authenticated users
    AuthenticatedRead,
    /// Allow full access to files in user's own folder (e.g., user123/*)
    UserFolderAccess,
    /// Allow full access to admin users only
    AdminFullAccess,
    /// Allow read-only access to users with specific role
    ReadOnlyForRole(String),
}
