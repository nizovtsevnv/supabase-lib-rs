# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2025-08-14

### 🚀 Major Features Added

#### 🔐 Complete Authentication Enhancement

- **OAuth Providers**: Full support for Google, GitHub, Discord, Apple, Twitter, Facebook, Microsoft, and LinkedIn
- **Phone Authentication**: SMS OTP sign-up, sign-in, and verification
- **Magic Links**: Passwordless email authentication with custom redirects
- **Anonymous Sign-in**: Temporary anonymous user sessions that can be converted to permanent accounts
- **Enhanced Password Recovery**: Improved password reset flows with custom redirects
- **Auth State Events**: Real-time authentication state change listeners with `onAuthStateChange`

#### 🛡️ Enhanced Error Handling & Context

- **Platform-Specific Error Context**: Automatic detection of WASM vs Native environments
- **HTTP Error Details**: Status codes, headers, response bodies, and retry information
- **Retry Logic**: Built-in retryability detection with suggested retry delays
- **Error Metadata**: Timestamp tracking and additional context information
- **Comprehensive Error Methods**: `is_retryable()`, `retry_after()`, `status_code()`, `context()`

### ✨ API Additions

#### New Authentication Methods

```rust
// OAuth sign-in
let response = client.auth()
    .sign_in_with_oauth(OAuthProvider::Google, Some(options))
    .await?;

// Phone authentication
let auth_response = client.auth()
    .sign_up_with_phone("+1234567890", "password", None)
    .await?;

// Magic links
client.auth()
    .sign_in_with_magic_link("user@example.com", Some(redirect_url), None)
    .await?;

// Anonymous sign-in
let auth_response = client.auth()
    .sign_in_anonymously(None)
    .await?;

// Auth event listeners
let handle = client.auth().on_auth_state_change(|event, session| {
    match event {
        AuthEvent::SignedIn => println!("User signed in!"),
        AuthEvent::SignedOut => println!("User signed out!"),
        AuthEvent::TokenRefreshed => println!("Token refreshed!"),
        _ => {}
    }
});
```

#### Enhanced Error Handling

```rust
// Platform-specific error context
match client.auth().sign_in_with_email_and_password("email", "password").await {
    Err(e) => {
        // Check if retryable
        if e.is_retryable() {
            if let Some(retry_after) = e.retry_after() {
                tokio::time::sleep(Duration::from_secs(retry_after)).await;
            }
        }

        // Get platform context
        if let Some(context) = e.context() {
            match &context.platform {
                Some(PlatformContext::Wasm { user_agent, available_apis, .. }) => {
                    println!("WASM: {:?}, APIs: {:?}", user_agent, available_apis);
                }
                Some(PlatformContext::Native { os_info, .. }) => {
                    println!("Native: {:?}", os_info);
                }
                _ => {}
            }
        }
    }
    Ok(response) => println!("Success: {:?}", response.user),
}
```

### 📚 Documentation Improvements

- **Comprehensive Module Documentation**: Extensive rustdoc with practical examples for all features
- **Platform-Specific Examples**: Separate examples for Native (Tokio) and WASM environments
- **Authentication Guide**: Complete guide covering all authentication methods with code examples
- **Error Handling Guide**: Best practices for error handling with platform context
- **Migration Examples**: Code examples showing migration from basic to enhanced features

### 🔧 Dependencies Added

- `urlencoding = "2.1"` - For OAuth URL parameter encoding
- `chrono` (with serde feature) - For timestamp handling in error context
- `web-sys` (optional) - For WASM platform detection

### 📈 API Coverage

- **Authentication**: ~95% (All major auth flows supported)
- **Database**: ~95% (Advanced operations from v0.3.0)
- **Storage**: ~85% (Full file management)
- **Realtime**: ~80% (WebSocket subscriptions)
- **Functions**: ~90% (Complete invoke functionality)
- **Error Handling**: ~98% (Comprehensive platform-aware error system)

### 🎯 Breaking Changes

**None** - v0.3.1 is fully backward compatible with v0.3.0. All existing code continues to work unchanged.

### 🚀 Performance Improvements

- **Optimized Error Construction**: Reduced allocation overhead in error handling paths
- **Efficient Event System**: Minimal-overhead authentication event listeners
- **Platform Detection Caching**: One-time platform context detection per client

### 🐛 Bug Fixes

- Fixed authentication state synchronization issues
- Improved error message consistency across all modules
- Enhanced WASM compatibility for authentication flows
- Fixed memory leaks in event listener cleanup

### 🔄 Version Compatibility

- **Rust**: 1.70.0 or higher
- **MSRV**: No change from v0.3.0
- **Supabase API**: Compatible with all current Supabase features

### 💡 Usage Examples

Complete working examples are available in the `/examples` directory:

- `auth_enhanced_example.rs` - All new authentication features
- `error_handling_example.rs` - Platform-aware error handling
- `oauth_example.rs` - OAuth provider integration
- `phone_auth_example.rs` - Phone authentication flow
- `anonymous_auth_example.rs` - Anonymous user management

### 🛣️ Next Steps (v0.3.2)

- Real-time authentication state synchronization
- Enhanced phone authentication with international support
- OAuth token refresh and management
- Advanced error recovery patterns

---

## [0.3.0] - Previous Release

### 🎉 Major Database Enhancements

This release dramatically expands database functionality with advanced querying capabilities, making the library suitable for complex production applications.

### ✨ Added

#### Database Advanced Operations
- **🔗 Logical Operators**: Complete support for `and()`, `or()`, `not()` query logic
  - Fluent builder pattern for complex filtering
  - PostgREST-compatible query parameter generation
  - Nested logical expressions support

- **🔗 Query Joins**: Full JOIN operation support
  - `inner_join()` and `left_join()` methods
  - `inner_join_as()` and `left_join_as()` with custom aliases
  - Automatic SELECT clause generation for joined tables

- **📦 Batch Operations**: Efficient bulk operations
  - `upsert()` method for insert-or-update logic
  - `bulk_insert()` for multiple records insertion
  - `bulk_upsert()` for bulk insert-or-update operations
  - Conflict resolution with `on_conflict()` method

- **🔧 Raw SQL Support**: Type-safe raw SQL execution
  - `raw_sql()` method for direct SQL queries via RPC
  - `prepared_statement()` for parameterized queries
  - `count_query()` for analytical operations

- **⚡ Database Transactions**: Complete transaction support
  - `transaction()` method for batch operations
  - `begin_transaction()` builder pattern for complex transactions
  - Support for INSERT, UPDATE, DELETE, SELECT, and RPC operations
  - Automatic rollback on errors

#### C FFI Foundation
- **🌍 C Foreign Function Interface**: Basic C-compatible bindings
  - `src/ffi.rs` module with C-compatible functions
  - `include/supabase.h` header file
  - Example C integration in `examples/c_usage/`
  - Support for client creation, auth, and database operations

#### Cross-Platform Releases
- **📦 Release Automation**: GitHub Actions for multi-platform builds
  - Automated builds for Linux (x86_64, ARM64)
  - macOS (x86_64, ARM64) support
  - Windows (x86_64, ARM64) support
  - WASM build optimization
  - Static/dynamic library generation

### 🔧 Changed

- **Enhanced Filter System**: Converted from simple struct to enum
  - `Filter::Simple` for basic column filters
  - `Filter::And`, `Filter::Or`, `Filter::Not` for logical operations
  - Recursive query parameter building

- **Improved SELECT Queries**: Enhanced QueryBuilder
  - Automatic join clause generation
  - Better column selection handling
  - Optimized query parameter building

- **Updated Examples**: All doctests now use proper async syntax
  - Removed `tokio_test::block_on` usage
  - Added proper error handling patterns
  - Fixed method names (`.values()` instead of `.set()`)

### 🐛 Fixed

- **Documentation**: All doctests now compile successfully
- **API Consistency**: Standardized method naming across builders
- **Memory Safety**: Improved FFI memory handling patterns
- **Type Safety**: Enhanced generic type support for all database operations

### 📊 Statistics

- **+800 Lines of Code** added to database module
- **15+ New Public Methods** for advanced database operations
- **22 New Unit Tests** with comprehensive coverage
- **All 85+ Tests Passing** including doctests
- **Zero Breaking Changes** to existing v0.2.0 API

### 🎯 API Coverage Progress

- **Database**: ~95% (All major operations + advanced features)
- **Auth**: ~85% (OAuth providers pending for v0.3.1)
- **Storage**: ~85% (Advanced features pending)
- **Functions**: ~90% (Complete invoke functionality)
- **Realtime**: ~80% (WebSocket subscriptions working)
- **FFI**: ~30% (Foundation established)

### 💡 Usage Examples

#### Logical Operators
```rust
// Complex filtering with AND/OR logic
let results = client.database()
    .from("users")
    .select("*")
    .and(|q| q.gte("age", "18").eq("status", "active"))
    .or(|q| q.eq("role", "admin").eq("role", "moderator"))
    .execute()
    .await?;
```
