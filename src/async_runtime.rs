//! Async runtime abstraction for cross-platform compatibility
//!
//! This module provides async synchronization primitives that work across
//! both native and WASM environments. The abstractions hide platform-specific
//! details while maintaining optimal performance for each target.
//!
//! ## Platform Support
//!
//! - **Native**: Uses `tokio::sync::RwLock` with full `Send + Sync` support
//! - **WASM**: Uses `futures_util::lock::Mutex` without `Send + Sync` bounds
//!
//! ## Usage
//!
//! ```rust,ignore
//! use supabase_lib_rs::async_runtime::RuntimeLock;
//! use std::collections::HashMap;
//!
//! # async fn example() {
//! // Create a cross-platform async lock
//! let data: RuntimeLock<HashMap<String, i32>> = RuntimeLock::new(HashMap::new());
//!
//! // Read access
//! {
//!     let reader = data.read().await;
//!     println!("Data has {} items", reader.len());
//! }
//!
//! // Write access
//! {
//!     let mut writer = data.write().await;
//!     writer.insert("key".to_string(), 42);
//! }
//! # }
//! ```

use std::future::Future;

/// Trait for async synchronization primitives on native platforms
///
/// This trait provides async read-write lock functionality with `Send + Sync`
/// bounds required for multi-threaded environments.
#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
#[async_trait::async_trait]
pub trait AsyncLock<T>: Send + Sync {
    /// Read guard type that implements `Deref<Target = T>`
    type Guard<'a>: std::ops::Deref<Target = T> + Send
    where
        T: 'a,
        Self: 'a;

    /// Write guard type that implements `DerefMut<Target = T>`
    type GuardMut<'a>: std::ops::DerefMut<Target = T> + Send
    where
        T: 'a,
        Self: 'a;

    /// Acquire a read lock
    ///
    /// # Examples
    /// ```rust,ignore
    /// # use supabase_lib_rs::async_runtime::{AsyncLock, TokioRwLock};
    /// # async fn example() {
    /// let lock = TokioRwLock::new(42);
    /// let guard = lock.read().await;
    /// println!("Value: {}", *guard);
    /// # }
    /// ```
    async fn read(&self) -> Self::Guard<'_>;

    /// Acquire a write lock
    ///
    /// # Examples
    /// ```rust,ignore
    /// # use supabase_lib_rs::async_runtime::{AsyncLock, TokioRwLock};
    /// # async fn example() {
    /// let lock = TokioRwLock::new(42);
    /// let mut guard = lock.write().await;
    /// *guard = 100;
    /// # }
    /// ```
    async fn write(&self) -> Self::GuardMut<'_>;
}

/// Trait for async synchronization primitives on WASM platforms
///
/// This trait is the same as the native version but without `Send + Sync` bounds
/// since WASM runs in a single-threaded environment.
#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
#[async_trait::async_trait(?Send)]
pub trait AsyncLock<T> {
    /// Read guard type that implements `Deref<Target = T>`
    type Guard<'a>: std::ops::Deref<Target = T>
    where
        T: 'a,
        Self: 'a;

    /// Write guard type that implements `DerefMut<Target = T>`
    type GuardMut<'a>: std::ops::DerefMut<Target = T>
    where
        T: 'a,
        Self: 'a;

    /// Acquire a read lock
    async fn read(&self) -> Self::Guard<'_>;

    /// Acquire a write lock
    async fn write(&self) -> Self::GuardMut<'_>;
}

/// Tokio-based implementation for native platforms
///
/// This provides full-featured async read-write locks using Tokio's primitives.
/// Supports multiple concurrent readers or one exclusive writer.
///
/// ## Features
///
/// - Multiple concurrent readers
/// - Exclusive writer access
/// - Fair scheduling to prevent writer starvation
/// - `Send + Sync` for multi-threaded use
///
/// ## Examples
///
/// ```rust,ignore
/// use supabase_lib_rs::async_runtime::{TokioRwLock, AsyncLock};
/// use std::collections::HashMap;
///
/// # async fn example() {
/// let shared_data = TokioRwLock::new(HashMap::<String, i32>::new());
///
/// // Multiple readers can access concurrently
/// let reader1 = shared_data.read().await;
/// let reader2 = shared_data.read().await;
/// println!("Both readers can access: {} items", reader1.len());
/// drop(reader1);
/// drop(reader2);
///
/// // Exclusive write access
/// let mut writer = shared_data.write().await;
/// writer.insert("key".to_string(), 42);
/// drop(writer);
/// # }
/// ```
#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
pub struct TokioRwLock<T>(tokio::sync::RwLock<T>);

#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
impl<T> TokioRwLock<T> {
    /// Create a new TokioRwLock
    ///
    /// # Examples
    /// ```rust,ignore
    /// use supabase_lib_rs::async_runtime::TokioRwLock;
    /// use std::collections::HashMap;
    ///
    /// let lock = TokioRwLock::new(HashMap::<String, i32>::new());
    /// ```
    pub fn new(value: T) -> Self {
        Self(tokio::sync::RwLock::new(value))
    }
}

#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
#[async_trait::async_trait]
impl<T: Send + Sync> AsyncLock<T> for TokioRwLock<T> {
    type Guard<'a>
        = tokio::sync::RwLockReadGuard<'a, T>
    where
        T: 'a;
    type GuardMut<'a>
        = tokio::sync::RwLockWriteGuard<'a, T>
    where
        T: 'a;

    async fn read(&self) -> Self::Guard<'_> {
        self.0.read().await
    }

    async fn write(&self) -> Self::GuardMut<'_> {
        self.0.write().await
    }
}

/// WASM implementation using futures-based locking
///
/// This provides async mutex functionality for WASM environments where
/// `Send + Sync` bounds are not available. Uses exclusive access only
/// (no concurrent readers).
///
/// ## Features
///
/// - Single-threaded async mutex
/// - Exclusive access for both reads and writes
/// - Compatible with WASM execution model
/// - Future-based implementation
///
/// ## Examples
///
/// ```rust,ignore
/// # #[cfg(target_arch = "wasm32")]
/// # async fn example() {
/// use supabase_lib_rs::async_runtime::{WasmRwLock, AsyncLock};
/// use std::collections::HashMap;
///
/// let shared_data = WasmRwLock::new(HashMap::<String, i32>::new());
///
/// // Read access (exclusive in WASM)
/// {
///     let reader = shared_data.read().await;
///     web_sys::console::log_1(&format!("Items: {}", reader.len()).into());
/// }
///
/// // Write access
/// {
///     let mut writer = shared_data.write().await;
///     writer.insert("key".to_string(), 42);
/// }
/// # }
/// ```
#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
pub struct WasmRwLock<T> {
    inner: futures_util::lock::Mutex<T>,
}

#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
impl<T> WasmRwLock<T> {
    /// Create a new WasmRwLock
    ///
    /// # Examples
    /// ```rust,ignore
    /// # #[cfg(target_arch = "wasm32")]
    /// # {
    /// use supabase_lib_rs::async_runtime::WasmRwLock;
    /// use std::collections::HashMap;
    ///
    /// let lock = WasmRwLock::new(HashMap::<String, i32>::new());
    /// # }
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            inner: futures_util::lock::Mutex::new(value),
        }
    }
}

/// Wrapper for mutex guard that implements Deref for read operations
#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
pub struct WasmReadGuard<'a, T>(futures_util::lock::MutexGuard<'a, T>);

#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
impl<'a, T> std::ops::Deref for WasmReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

/// Wrapper for mutex guard that implements DerefMut for write operations
#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
pub struct WasmWriteGuard<'a, T>(futures_util::lock::MutexGuard<'a, T>);

#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
impl<'a, T> std::ops::Deref for WasmWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
impl<'a, T> std::ops::DerefMut for WasmWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
#[async_trait::async_trait(?Send)]
impl<T> AsyncLock<T> for WasmRwLock<T> {
    type Guard<'a>
        = WasmReadGuard<'a, T>
    where
        T: 'a;
    type GuardMut<'a>
        = WasmWriteGuard<'a, T>
    where
        T: 'a;

    async fn read(&self) -> Self::Guard<'_> {
        WasmReadGuard(self.inner.lock().await)
    }

    async fn write(&self) -> Self::GuardMut<'_> {
        WasmWriteGuard(self.inner.lock().await)
    }
}

/// Type alias for the appropriate lock implementation based on target platform
///
/// This automatically selects the correct lock implementation:
/// - **Native**: `TokioRwLock<T>` with full read-write semantics
/// - **WASM**: `WasmRwLock<T>` with exclusive access only
///
/// ## Examples
///
/// ```rust,ignore
/// use supabase_lib_rs::async_runtime::RuntimeLock;
/// use std::collections::HashMap;
///
/// # async fn example() {
/// // Automatically uses the right lock for your platform
/// let data: RuntimeLock<HashMap<String, i32>> = RuntimeLock::new(HashMap::new());
///
/// // Works the same on both platforms
/// let reader = data.read().await;
/// println!("Platform-agnostic read access");
/// drop(reader);
///
/// let mut writer = data.write().await;
/// writer.insert("key".to_string(), 42);
/// # }
/// ```
#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
pub type RuntimeLock<T> = TokioRwLock<T>;

/// Type alias for WASM runtime locks
#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
pub type RuntimeLock<T> = WasmRwLock<T>;

/// Spawn a task on the appropriate runtime
#[cfg(all(feature = "realtime", not(target_arch = "wasm32")))]
#[allow(dead_code)]
pub fn spawn_task<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(future)
}

#[cfg(all(feature = "realtime", target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn spawn_task<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}
