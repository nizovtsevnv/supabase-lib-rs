# 🧪 Testing Guide for supabase-lib-rs

This document describes the comprehensive testing system for the Supabase Rust client library.

## 📋 Overview

The project uses a multi-tier testing approach:

1. **Unit Tests** - Fast, isolated component tests
2. **Documentation Tests** - Validate code examples in docs
3. **Integration Tests** - Test against real Supabase API
4. **End-to-End Tests** - Full workflow scenarios

## 🚀 Quick Start

### Run All Tests

```bash
# Start local Supabase and run all tests
just test-all

# Or step-by-step:
just supabase-start  # Start local Supabase
just test            # Unit + doc tests
just test-integration # Integration tests
```

### Run Individual Test Types

```bash
# Unit tests only (fast)
just test-unit

# Documentation tests only
just test-doc

# Integration tests only (requires Supabase)
just test-integration
```

## 🐳 Docker/Podman Setup

### Start Local Supabase

The project includes a complete Docker Compose setup:

```bash
# Start all services (auto-detects Docker/Podman)
just supabase-start

# Alternative: Use Supabase CLI (if installed)
just supabase-cli-start
```

### Available Services

Once started, you'll have access to:

| Service      | URL                                        | Description                        |
| ------------ | ------------------------------------------ | ---------------------------------- |
| **Studio**   | http://localhost:54323                     | Web UI for database management     |
| **API**      | http://localhost:54321                     | REST API, Auth, Realtime endpoints |
| **Database** | localhost:54322                            | PostgreSQL database                |
| **Realtime** | ws://localhost:54321/realtime/v1/websocket | WebSocket endpoint                 |

### Environment Variables

Tests automatically use these environment variables:

```bash
SUPABASE_URL=http://localhost:54321
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## 🧪 Test Categories

### Unit Tests (`just test-unit`)

Fast, isolated tests for individual components:

```bash
# Example unit test
#[test]
fn test_client_creation() {
    let client = Client::new("http://localhost:54321", "test-key");
    assert!(client.is_ok());
}
```

**Location:** `src/*/mod.rs` (inline `#[cfg(test)]` modules)

### Documentation Tests (`just test-doc`)

Validates all code examples in documentation:

````rust
/// Create a Supabase client
/// ```
/// use supabase::Client;
///
/// let client = Client::new("http://localhost:54321", "your-anon-key")?;
/// # Ok::<(), supabase::Error>(())
/// ```
````

**Location:** `src/lib.rs` and other source files with `///` docs

### Integration Tests (`just test-integration`)

Test individual modules against real Supabase:

- ✅ **Client creation and health checks**
- ✅ **Auth module initialization**
- ✅ **Database query builder**
- ✅ **Storage bucket operations**
- ✅ **Realtime connection lifecycle**

**Location:** `tests/integration_tests.rs`

### End-to-End Tests

Full workflow scenarios:

- ✅ **Preflight checks** (health + version)
- ✅ **Auth + Database flow** (combined workflows)
- ✅ **Storage workflow** (create bucket → upload → delete)

**Location:** `tests/integration_tests.rs` (E2E test functions)

## 🔄 Test Safety & Skipping

### Automatic Skipping

Integration tests **automatically skip** when Supabase is not available:

```rust
#[tokio::test]
async fn integration_test() {
    if skip_if_no_supabase() {
        println!("⏭️ Skipping - Supabase not configured");
        return;
    }

    // Test code runs only if Supabase is available
}
```

### CI/CD Safety

This design makes tests safe for CI/CD environments:

- Unit tests always run
- Integration tests skip gracefully without Supabase
- No external dependencies required for basic testing

## 🛠️ Docker Management Commands

### Basic Operations

```bash
just supabase-start    # Start all services
just supabase-stop     # Stop all services
just supabase-restart  # Restart all services
just supabase-status   # Show container status
```

### Debugging & Logs

```bash
just supabase-logs           # All logs
just supabase-logs auth      # Auth service logs
just supabase-logs db        # Database logs
just supabase-logs realtime  # Realtime logs
```

### Data Management

```bash
just supabase-clean    # Remove all data and volumes
```

### Health Checks

```bash
just supabase-ensure-running  # Auto-start if not running
```

## 🔧 Configuration

### Custom Environment

For remote Supabase testing:

```bash
export SUPABASE_URL="https://your-project.supabase.co"
export SUPABASE_ANON_KEY="your-anon-key"
export SUPABASE_SERVICE_ROLE_KEY="your-service-role-key"

# Optional: Test user credentials
export TEST_USER_EMAIL="test@example.com"
export TEST_USER_PASSWORD="testpassword123"
```

### Local Development

Copy `.env.example` to `.env` and customize:

```bash
cp .env.example .env
# Edit .env as needed
```

## 🚨 Troubleshooting

### Common Issues

**Docker not found:**

```
❌ No Docker/Podman Compose found
   Install Docker Compose or Podman
```

→ Install Docker Desktop or Podman

**Port conflicts:**

```
Error: port 54321 already in use
```

→ Change ports in `.env` file or stop conflicting services

**Integration tests fail:**

```
⚠️ Integration tests require Supabase configuration
   Run: just supabase-start
```

→ Start Supabase first

### Debug Mode

Run tests with verbose output:

```bash
RUST_LOG=debug cargo test --test integration_tests -- --nocapture
```

### Container Issues

Check container status:

```bash
just supabase-status
just supabase-logs
```

Reset everything:

```bash
just supabase-clean
just supabase-start
```

## 📊 Test Coverage

Generate coverage reports:

```bash
just coverage
```

## 🎯 Best Practices

### Writing Integration Tests

1. **Always check availability:**

   ```rust
   if skip_if_no_supabase() { return; }
   ```

2. **Handle client creation gracefully:**

   ```rust
   let client = match create_test_client() {
       Some(client) => client,
       None => { println!("⏭️ Skipping"); return; }
   };
   ```

3. **Use descriptive output:**
   ```rust
   println!("✅ Test passed");
   println!("⚠️ Warning condition");
   println!("❌ Test failed");
   ```

### Local Development

1. Always start Supabase before integration tests
2. Use `just test-all` for complete validation
3. Check logs if tests behave unexpectedly
4. Clean data between major test changes

### CI/CD Integration

The test system is designed for CI/CD:

```yaml
# GitHub Actions example
- run: cargo test --lib # Unit tests
- run: cargo test --doc # Doc tests
- run: docker-compose up -d # Start Supabase
- run: sleep 10 # Wait for services
- run: cargo test --test integration_tests # Integration tests
```

---

## 🎉 Success!

With this testing system, you have:

- ✅ **Comprehensive coverage** across all test types
- ✅ **Local development** with Docker/Podman
- ✅ **CI/CD ready** with automatic skipping
- ✅ **Easy debugging** with detailed logs
- ✅ **Production confidence** through real API testing

Happy testing! 🚀
