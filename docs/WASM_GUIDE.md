# WASM Integration Guide

This guide explains how to use the Supabase Rust client library in WebAssembly (WASM) environments, particularly with frameworks like Dioxus.

## Features Available in WASM

✅ **Authentication**: Sign in, sign up, sign out, session management
✅ **Database**: CRUD operations, queries, filters, ordering
✅ **Storage**: File upload, download, management (using simple body uploads)
✅ **Realtime**: WebSocket subscriptions for live updates

## Quick Start

### 1. Add Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
supabase-lib-rs = "0.2.0"

# For WASM applications
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = "0.3"

# For Dioxus applications
dioxus = "0.6"
dioxus-web = "0.6"
```

### 2. Initialize Client

```rust
use supabase::Client;

let client = Client::new(
    "https://your-project.supabase.co",
    "your-anon-key"
)?;
```

### 3. Basic Usage

```rust
// Authentication
let user = client.auth()
    .sign_in_with_email_and_password("user@example.com", "password")
    .await?;

// Database operations
let todos: Vec<Todo> = client.database()
    .from("todos")
    .select("*")
    .execute()
    .await?;

// Realtime subscriptions
client.realtime().connect().await?;
let subscription_id = client.realtime()
    .channel("todos")
    .table("todos")
    .subscribe(|message| {
        console_log!("Update: {:?}", message);
    })
    .await?;
```

## Dioxus Integration

### Complete Todo App Example

See `examples/dioxus_example.rs` for a full implementation of a todo app using Dioxus and Supabase.

Key patterns:

```rust
#[component]
fn App() -> Element {
    // Initialize Supabase client
    let supabase = use_signal(|| {
        SupabaseService::new("your-url", "your-key").ok()
    });

    // Reactive state
    let todos = use_signal(|| Vec::<Todo>::new());
    let authenticated = use_signal(|| false);

    // Effects for data loading
    use_effect(move || {
        if authenticated() && supabase().is_some() {
            spawn(async move {
                let todos_data = supabase().unwrap().get_todos().await?;
                todos.set(todos_data);
            });
        }
    });

    // Event handlers
    let handle_create_todo = move |title: String| {
        spawn(async move {
            let new_todo = supabase().unwrap().create_todo(&title).await?;
            todos.with_mut(|t| t.push(new_todo));
        });
    };

    rsx! {
        // Your UI components
    }
}
```

## Building for WASM

### With Dioxus CLI

```bash
# Install Dioxus CLI
cargo install dioxus-cli

# Create a new project
dx new my-supabase-app
cd my-supabase-app

# Add supabase dependency to Cargo.toml

# Build for web
dx build --release --target web

# Serve locally
dx serve --target web
```

### Manual WASM Build

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Build
cargo build --target wasm32-unknown-unknown --release

# Use wasm-bindgen to generate JS bindings
wasm-bindgen target/wasm32-unknown-unknown/release/your_app.wasm \
  --out-dir pkg --target web
```

## API Differences in WASM

### Storage Module

In WASM, file uploads use simple body uploads instead of multipart:

```rust
// Native: Uses multipart upload
client.storage().upload("bucket", "path", file_bytes, None).await?;

// WASM: Uses simple body upload (same API, different implementation)
client.storage().upload("bucket", "path", file_bytes, None).await?;
```

### Realtime Module

WebSocket implementation uses the browser's native WebSocket API:

```rust
// Works identically on both native and WASM
let subscription = client.realtime()
    .channel("table_changes")
    .table("todos")
    .event(RealtimeEvent::Insert)
    .subscribe(|message| {
        // Handle realtime updates
    })
    .await?;
```

## Error Handling

WASM errors are handled the same way as native:

```rust
match client.auth().sign_in_with_email_and_password(email, password).await {
    Ok(response) => {
        // Success
    }
    Err(supabase::Error::Auth(e)) => {
        // Authentication error
    }
    Err(supabase::Error::Network(e)) => {
        // Network error
    }
    Err(e) => {
        // Other errors
    }
}
```

## Performance Considerations

1. **Bundle Size**: The WASM build is optimized for size. Unused modules are automatically excluded.

2. **Async Operations**: Use `spawn_local` for fire-and-forget operations:

   ```rust
   wasm_bindgen_futures::spawn_local(async move {
       let _ = client.database().from("analytics").insert(event).execute().await;
   });
   ```

3. **Realtime**: WebSocket connections are managed automatically but consider connection lifecycle in SPAs.

## Security Notes

1. **API Keys**: Only use anon keys in WASM/frontend code. Never expose service role keys.

2. **RLS**: Enable Row Level Security (RLS) in your Supabase database for data protection.

3. **HTTPS**: Always use HTTPS URLs for production Supabase projects.

## Troubleshooting

### Build Issues

1. **Missing clang**: Use the Nix development environment:

   ```bash
   nix develop -c cargo build --target wasm32-unknown-unknown
   ```

2. **Dependencies**: Ensure all WASM-specific dependencies are included in your `Cargo.toml`.

### Runtime Issues

1. **CORS**: Configure CORS settings in your Supabase dashboard for your domain.

2. **WebSocket**: Realtime subscriptions require secure WebSocket connections (wss://) in production.

3. **Console Logging**: Use `web_sys::console::log_1()` for debugging in WASM:
   ```rust
   web_sys::console::log_1(&format!("Debug: {:?}", data).into());
   ```

## Examples

- `examples/wasm_example.rs` - Basic WASM usage
- `examples/dioxus_example.rs` - Complete Dioxus todo app
- `examples/basic_usage.rs` - Cross-platform patterns

## Framework Integration

### Dioxus

Complete support with reactive state management.

### Yew

Compatible with Yew's async patterns using `spawn_local`.

### Leptos

Works with Leptos's reactive system and SSR.

### Vanilla WASM

Can be used with any WASM setup using `wasm-bindgen`.

For more examples and updates, see the [project repository](https://github.com/nizovtsevnv/supabase-lib-rs).
