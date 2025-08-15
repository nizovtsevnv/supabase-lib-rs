# Contributing to Supabase Rust Client

Thank you for your interest in contributing to the Supabase Rust client library! This document outlines the contribution process, coding standards, and architectural principles we follow.

## 🎯 Core Principles

This project emphasizes **simplicity**, **efficiency**, and **maintainability**:

- **Minimize complexity** - Prefer simple, clear solutions over clever ones
- **One responsibility per module** - Each file should have a single, well-defined purpose
- **Efficiency first** - Code should be performant on both native and WASM targets
- **Cross-platform compatibility** - Support native (desktop/server) and WASM (web) seamlessly

## 🚀 Quick Start

### Prerequisites

- **Nix** (recommended) - For reproducible development environment
- **Rust 2024 edition** with latest stable toolchain
- **Git** for version control

### Setup

```bash
# Clone the repository
git clone https://github.com/nizovtsevnv/supabase-lib-rs.git
cd supabase-lib-rs

# Enter reproducible development environment
nix develop

# Alternative without Nix (ensure you have Rust 1.75+ installed)
rustup update stable
rustup target add wasm32-unknown-unknown
```

## 🛠️ Development Workflow

### 1. Before You Start

1. **Check existing issues** - Look for related work or discussions
2. **Create an issue** - For significant changes, discuss the approach first
3. **Fork the repository** - Work on your own fork for external contributions

### 2. Making Changes

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Make your changes following the coding standards below
# Ensure you run checks frequently during development

# Test your changes
nix develop -c cargo check
nix develop -c cargo test
nix develop -c cargo check --target wasm32-unknown-unknown
```

### 3. Pre-Commit Validation

**CRITICAL**: These checks MUST pass before committing:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Lint with zero warnings
cargo clippy -- -D warnings

# Run all tests
cargo test

# Build release version
cargo build --release

# Verify WASM compatibility
cargo check --target wasm32-unknown-unknown

# Optional: Run Nix build (if using Nix)
nix build
```

### 4. Submitting Changes

```bash
# Commit with meaningful message
git commit -m "feat: add WebSocket reconnection logic"

# Push to your fork
git push origin feature/your-feature-name

# Create Pull Request with:
# - Clear description of changes
# - Test plan
# - WASM compatibility confirmation
```

## 📋 Coding Standards

### Code Style

**Follow Rust conventions strictly:**

```rust
// ✅ Good: Clear, descriptive names
pub struct DatabaseClient {
    http_client: Arc<HttpClient>,
    config: Arc<DatabaseConfig>,
}

impl DatabaseClient {
    pub async fn execute_query(&self, query: &str) -> Result<QueryResponse> {
        // Implementation
    }
}

// ❌ Bad: Unclear names, complex structure
pub struct DbC {
    c: Arc<HttpClient>,
    cfg: Arc<DatabaseConfig>,
    cache: HashMap<String, CachedResult>,
    stats: QueryStats,
}
```

**Naming conventions:**

- `snake_case` - Functions, variables, modules
- `PascalCase` - Structs, enums, traits
- `SCREAMING_SNAKE_CASE` - Constants
- **Meaningful names** that reflect purpose

### Error Handling

```rust
// ✅ Use Result<T, E> for fallible operations
pub async fn fetch_user(&self, id: u64) -> Result<User> {
    let response = self.http_client
        .get(&format!("/users/{}", id))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::not_found("User not found"));
    }

    Ok(response.json().await?)
}

// ❌ Don't panic in library code
pub fn get_user(&self, id: u64) -> User {
    self.users.get(&id).expect("User must exist") // Don't do this!
}
```

### Cross-Platform Code

```rust
// ✅ Platform-specific implementations
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::WebSocketStream;

#[cfg(target_arch = "wasm32")]
use web_sys::WebSocket;

// ✅ Conditional compilation for different features
#[cfg(not(target_arch = "wasm32"))]
impl Storage {
    pub async fn upload_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // Native file system access
    }
}

#[cfg(target_arch = "wasm32")]
impl Storage {
    pub async fn upload_file(&self, file: &[u8]) -> Result<()> {
        // WASM-compatible upload
    }
}
```

### Module Organization

**One responsibility per file:**

```
src/
├── lib.rs          # Public API exports
├── client.rs       # Main client struct
├── error.rs        # Error types
├── types.rs        # Common types
├── auth.rs         # Authentication module
├── database.rs     # Database operations
├── storage.rs      # File storage
└── realtime/       # Realtime module
    ├── mod.rs      # Public interface
    ├── native.rs   # Native WebSocket implementation
    └── wasm.rs     # WASM WebSocket implementation
```

## 🧪 Testing Standards

### Test Organization

```
tests/
├── integration/        # Integration tests
│   ├── auth_tests.rs
│   ├── database_tests.rs
│   ├── storage_tests.rs
│   ├── realtime_tests.rs
│   ├── e2e_*.rs       # End-to-end tests
│   └── common/
│       └── mod.rs     # Test utilities
```

### Writing Tests

```rust
// ✅ Comprehensive test coverage
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_user_authentication_success() {
        // Arrange
        let mut server = Server::new_async().await;
        let mock = server.mock("POST", "/auth/v1/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"access_token": "token123"}"#)
            .create_async()
            .await;

        let client = Client::new(&server.url(), "test-key").unwrap();

        // Act
        let result = client.auth()
            .sign_in_with_email_and_password("test@example.com", "password")
            .await;

        // Assert
        assert!(result.is_ok());
        let auth_response = result.unwrap();
        assert_eq!(auth_response.access_token, "token123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_user_authentication_invalid_credentials() {
        // Test error cases too
    }
}
```

### Test Requirements

- **Minimum 80% coverage** for public APIs
- **100% coverage** for critical security functions
- **Both positive and negative test cases**
- **Mock external dependencies** (don't hit real Supabase in tests)
- **WASM compatibility tests** where applicable

## 🏗️ Architecture Guidelines

### Adding New Features

1. **Start simple** - Implement the minimal viable feature first
2. **Consider both platforms** - Ensure native and WASM compatibility
3. **Follow existing patterns** - Look at similar modules for guidance
4. **Document public APIs** - Use rustdoc for all public functions

### Performance Considerations

```rust
// ✅ Efficient async operations
pub async fn batch_insert<T>(&self, items: &[T]) -> Result<Vec<T>>
where
    T: Serialize + DeserializeOwned,
{
    // Batch operations instead of individual requests
    let payload = serde_json::json!({ "items": items });
    let response = self.http_client
        .post("/batch_insert")
        .json(&payload)
        .send()
        .await?;

    Ok(response.json().await?)
}

// ❌ Inefficient: Multiple individual requests
pub async fn insert_multiple<T>(&self, items: &[T]) -> Result<Vec<T>> {
    let mut results = Vec::new();
    for item in items {
        results.push(self.insert(item).await?); // Don't do this!
    }
    Ok(results)
}
```

### Memory Management

- **Use `Arc<T>` for shared ownership** (client configurations, etc.)
- **Prefer borrowing over cloning** where possible
- **Avoid unnecessary allocations** in hot paths
- **Use `Bytes` for binary data** instead of `Vec<u8>`

### Error Design

```rust
// ✅ Structured error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Authentication failed: {message}")]
    Auth { message: String },

    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Storage error: {message}")]
    Storage { message: String },
}

// ✅ Helpful error context
impl Error {
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth { message: message.into() }
    }
}
```

## 📚 Documentation Standards

### Public API Documentation

````rust
/// Executes a database query with the specified parameters.
///
/// This method sends a SQL query to the Supabase database and returns
/// the results deserialized into the specified type.
///
/// # Arguments
///
/// * `query` - The SQL query string to execute
/// * `params` - Optional parameters for the query
///
/// # Returns
///
/// Returns `Ok(Vec<T>)` with the query results, or `Err(Error)` if the
/// query fails or the response cannot be deserialized.
///
/// # Example
///
/// ```rust
/// use supabase::Client;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize)]
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("url", "key")?;
/// let users: Vec<User> = client
///     .database()
///     .from("users")
///     .select("id, name")
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// * `Error::Http` - Network or HTTP-level errors
/// * `Error::Database` - Database query errors
/// * `Error::Json` - JSON deserialization errors
pub async fn execute<T>(&self) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    // Implementation
}
````

### Internal Documentation

- **Comment complex algorithms** and business logic
- **Explain platform-specific differences**
- **Document performance considerations**
- **Keep comments up-to-date** with code changes

## 🔍 Code Review Process

### For Reviewers

**Focus on:**

1. **Correctness** - Does the code work as intended?
2. **Simplicity** - Is this the simplest solution?
3. **Performance** - Are there obvious inefficiencies?
4. **Cross-platform** - Does it work on both native and WASM?
5. **Tests** - Are there adequate tests?
6. **Documentation** - Is the public API documented?

### For Contributors

**Prepare for review:**

1. **Self-review first** - Check your own code thoroughly
2. **Test thoroughly** - Ensure all platforms work
3. **Keep changes focused** - One feature/fix per PR
4. **Write clear descriptions** - Explain what and why
5. **Be responsive** - Address feedback promptly

## 🐛 Reporting Issues

### Bug Reports

**Include:**

- **Rust version** and target platform (native/WASM)
- **Library version**
- **Minimal reproduction case**
- **Expected vs actual behavior**
- **Error messages** (full stack traces)

### Feature Requests

**Include:**

- **Use case description** - What are you trying to achieve?
- **Proposed API** - How should it work?
- **Alternatives considered** - What other solutions did you try?
- **Cross-platform considerations** - How should it work on WASM?

## 📖 Resources

- **[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)**
- **[Supabase API Documentation](https://supabase.com/docs)**
- **[WebAssembly in Rust](https://rustwasm.github.io/book/)**
- **[Async Rust](https://rust-lang.github.io/async-book/)**

## 🤝 Community

- **Be respectful** and inclusive
- **Help newcomers** learn the codebase
- **Share knowledge** and best practices
- **Focus on the code**, not the person

## 🚀 Release & Publishing (For Maintainers)

### Automated Publishing

This project uses GitHub Actions for automated publishing:

- **📦 crates.io**: Automatically publishes on GitHub releases
- **🐍 PyPI**: Cross-platform wheels built on releases  
- **📚 docs.rs**: Documentation auto-generated from crates.io
- **🏗️  Cross-platform libraries**: Built for Linux, macOS, Windows (x86_64 + ARM64)

### Release Process

1. **Update version** in `Cargo.toml` and `pyproject.toml`
2. **Update CHANGELOG.md** with new version details
3. **Run full test suite**: `just check`
4. **Create GitHub Release** with tag `vX.Y.Z`
5. **Automatic publishing** triggers for all platforms

### Manual Publishing (Emergency Use)

For manual releases, use the GitHub Actions workflows:

**🦀 Rust (crates.io):**
1. Go to **Actions** → **Manual Publish to crates.io**
2. Click **Run workflow**
3. Optionally specify version or run dry-run first
4. Monitor the workflow for success

**🐍 Python (PyPI):**
1. Go to **Actions** → **Manual Publish to PyPI**
2. Click **Run workflow**  
3. Optionally specify version or run dry-run (TestPyPI)
4. Monitor the workflow for success

### Version Management

- Follow semantic versioning: `MAJOR.MINOR.PATCH`
- Version bumps trigger automatic releases when tagged (`git tag v0.5.1`)
- Always update `CHANGELOG.md` before releases
- Test thoroughly on all target platforms

---

**Thank you for contributing to making Supabase accessible to the Rust community!** 🦀
