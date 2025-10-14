# Configuration Guide

This guide covers all configuration options for the Supabase Rust Client library.

## Environment Variables

The library can be configured using environment variables. This is the most common configuration method.

### Setup

Copy `.env.example` to `.env` and fill in your actual values:

```bash
cp .env.example .env
```

### Required Variables

- `SUPABASE_URL` - Your Supabase project URL
- `SUPABASE_ANON_KEY` - Your Supabase anonymous key

### Optional Variables

- `SUPABASE_SERVICE_ROLE_KEY` - Service role key for admin operations
- `RUST_LOG` - Log level (debug, info, warn, error)
- `RUST_BACKTRACE` - Enable backtrace (0, 1, full)

## Getting Your Supabase Keys

1. Go to your [Supabase Dashboard](https://supabase.com/dashboard)
2. Select your project
3. Navigate to Settings > API
4. Copy the keys:
   - **Project URL** → `SUPABASE_URL`
   - **anon public** → `SUPABASE_ANON_KEY`
   - **service_role** → `SUPABASE_SERVICE_ROLE_KEY` (keep this secret!)

## Programmatic Configuration

### Basic Configuration

```rust
use supabase::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new(
        "https://your-project.supabase.co",
        "your-anon-key"
    )?;

    Ok(())
}
```

### Service Role Configuration

```rust
use supabase::Client;

let admin_client = Client::new_with_service_role(
    "https://your-project.supabase.co",
    "your-anon-key",
    "your-service-role-key"
)?;
```

### Advanced Configuration

```rust
use supabase::{Client, types::*};
use std::collections::HashMap;

let config = SupabaseConfig {
    url: "https://your-project.supabase.co".to_string(),
    key: "your-anon-key".to_string(),
    service_role_key: None,
    http_config: HttpConfig {
        timeout: 30,
        connect_timeout: 10,
        max_redirects: 5,
        default_headers: HashMap::new(),
    },
    auth_config: AuthConfig {
        auto_refresh_token: true,
        refresh_threshold: 300,
        persist_session: true,
        storage_key: "supabase.auth.token".to_string(),
    },
    database_config: DatabaseConfig {
        schema: "public".to_string(),
        max_retries: 3,
        retry_delay: 1000,
    },
    storage_config: StorageConfig {
        default_bucket: Some("uploads".to_string()),
        upload_timeout: 300,
        max_file_size: 50 * 1024 * 1024, // 50MB
    },
};

let client = Client::new_with_config(config)?;
```

## Configuration Options

### HTTP Configuration

| Option            | Type                      | Default | Description                      |
| ----------------- | ------------------------- | ------- | -------------------------------- |
| `timeout`         | `u64`                     | `30`    | Request timeout in seconds       |
| `connect_timeout` | `u64`                     | `10`    | Connection timeout in seconds    |
| `max_redirects`   | `usize`                   | `5`     | Maximum number of redirects      |
| `default_headers` | `HashMap<String, String>` | `{}`    | Default headers for all requests |

### Authentication Configuration

| Option               | Type     | Default                 | Description                           |
| -------------------- | -------- | ----------------------- | ------------------------------------- |
| `auto_refresh_token` | `bool`   | `true`                  | Automatically refresh expired tokens  |
| `refresh_threshold`  | `u64`    | `300`                   | Refresh token before expiry (seconds) |
| `persist_session`    | `bool`   | `true`                  | Persist session across app restarts   |
| `storage_key`        | `String` | `"supabase.auth.token"` | Storage key for session data          |

### Database Configuration

| Option        | Type     | Default    | Description                                |
| ------------- | -------- | ---------- | ------------------------------------------ |
| `schema`      | `String` | `"public"` | Default database schema                    |
| `max_retries` | `u32`    | `3`        | Maximum retry attempts for failed requests |
| `retry_delay` | `u64`    | `1000`     | Delay between retries (milliseconds)       |

### Storage Configuration

| Option           | Type             | Default | Description                           |
| ---------------- | ---------------- | ------- | ------------------------------------- |
| `default_bucket` | `Option<String>` | `None`  | Default bucket for storage operations |
| `upload_timeout` | `u64`            | `300`   | Upload timeout in seconds             |
| `max_file_size`  | `usize`          | `50MB`  | Maximum file size for uploads         |

## Feature Flags

Control which features are included in your build:

```toml
[dependencies]
supabase-lib-rs = { version = "0.5.2", features = ["auth", "database", "storage"] }
```

### Available Features

| Feature     | Description             | Dependencies              |
| ----------- | ----------------------- | ------------------------- |
| `auth`      | Authentication module   | `jsonwebtoken`, `chrono`  |
| `database`  | Database operations     | `serde_json`              |
| `storage`   | File storage operations | `mime`                    |
| `functions` | Edge functions          | `serde_json`              |
| `realtime`  | Real-time subscriptions | `tokio-tungstenite`       |
| `native`    | Native platform support | `tokio`                   |
| `wasm`      | WebAssembly support     | `web-sys`, `wasm-bindgen` |

### Platform-Specific Features

#### Native Features

- File system access for session storage
- Full WebSocket support
- Native HTTP client with connection pooling

#### WASM Features

- Browser LocalStorage integration
- Web APIs for WebSocket and HTTP
- Optimized bundle size

## Logging Configuration

### Environment Variables

```bash
# Enable debug logging
RUST_LOG=debug

# Enable specific module logging
RUST_LOG=supabase=debug,reqwest=info

# Enable full backtraces
RUST_BACKTRACE=full
```

### Programmatic Logging

```rust
use tracing_subscriber;

// Initialize logging
tracing_subscriber::fmt::init();

// Use structured logging
tracing::info!("Client initialized successfully");
tracing::debug!(url = %client.url(), "Connected to Supabase");
```

## Error Handling Configuration

### Retry Configuration

```rust
use supabase::types::*;

let config = SupabaseConfig {
    // ... other config
    database_config: DatabaseConfig {
        max_retries: 5,           // Retry up to 5 times
        retry_delay: 2000,        // Wait 2 seconds between retries
        // ... other options
    },
};
```

### Custom Error Handlers

```rust
use supabase::{Client, Error};

let client = Client::new(url, key)?;

// Handle specific error types
match client.auth().sign_in_with_password("email", "password").await {
    Ok(response) => println!("Signed in: {:?}", response.user),
    Err(Error::Auth { source, context }) => {
        println!("Auth error: {} ({})", source, context);
    },
    Err(e) => println!("Other error: {}", e),
}
```

## Security Configuration

### Session Encryption

```rust
use supabase::types::*;

let config = SupabaseConfig {
    auth_config: AuthConfig {
        encrypt_session: true,     // Enable session encryption
        encryption_key: Some("your-32-char-encryption-key".to_string()),
        // ... other options
    },
    // ... other config
};
```

### Network Security

```rust
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("X-Custom-Header".to_string(), "value".to_string());

let config = SupabaseConfig {
    http_config: HttpConfig {
        default_headers: headers,
        // ... other options
    },
    // ... other config
};
```

## Development vs Production

### Development Configuration

```rust
// Development - more verbose logging, longer timeouts
let dev_config = SupabaseConfig {
    http_config: HttpConfig {
        timeout: 60,              // Longer timeout for debugging
        // ... other options
    },
    // ... other config
};
```

### Production Configuration

```rust
// Production - optimized for performance and reliability
let prod_config = SupabaseConfig {
    http_config: HttpConfig {
        timeout: 15,              // Shorter timeout
        max_redirects: 2,         // Fewer redirects
        // ... other options
    },
    auth_config: AuthConfig {
        refresh_threshold: 600,   // Refresh tokens earlier
        // ... other options
    },
    // ... other config
};
```

## Troubleshooting

### Common Issues

#### Connection Timeouts

```rust
// Increase timeout for slow networks
let config = SupabaseConfig {
    http_config: HttpConfig {
        timeout: 60,
        connect_timeout: 20,
        // ... other options
    },
    // ... other config
};
```

#### Authentication Issues

```bash
# Enable auth debugging
RUST_LOG=supabase::auth=debug

# Check your keys
echo $SUPABASE_URL
echo $SUPABASE_ANON_KEY
```

#### CORS Issues (WASM)

Make sure your Supabase project allows your domain:

1. Go to Authentication > Settings
2. Add your domain to "Site URL"
3. Add your domain to "Additional Redirect URLs"

### Debug Mode

```rust
use supabase::{Client, types::*};

let config = SupabaseConfig {
    // Enable detailed logging
    debug: true,
    // ... other config
};
```

## Related Documentation

- [Architecture Guide](ARCHITECTURE.md)
- [WebAssembly Guide](WASM_GUIDE.md)
- [Testing Guide](../TESTING.md)
- [Contributing Guide](../CONTRIBUTING.md)
