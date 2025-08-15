# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-08-15

### 🚀 Major Features Added

#### 🔐 Advanced Session Management

- **Session Persistence**: Cross-tab session synchronization with automatic state management
- **Platform-aware Storage**: Intelligent storage backend selection (localStorage for WASM, filesystem for Native)
- **Cross-tab Synchronization**: Real-time session sync using BroadcastChannel (WASM) and filesystem IPC (Native)
- **Session Encryption**: AES-256-GCM encryption for secure session storage with key derivation
- **Session Monitoring**: Real-time session state tracking with event-driven architecture
- **Session Events**: Comprehensive event system (Created, Updated, Accessed, Destroyed, etc.)

#### 🏗️ Modular Architecture

- **SessionManager**: Centralized session management with configuration-driven setup
- **SessionStorage Trait**: Pluggable storage backends (Memory, LocalStorage, FileSystem, Encrypted)
- **CrossTabChannel Trait**: Platform-specific cross-tab communication abstractions
- **Device Detection**: Browser fingerprinting (WASM) and system information gathering (Native)

#### 🛡️ Security Enhancements

- **Session Encryption**: Optional AES-256-GCM encryption for sensitive session data
- **Key Management**: Secure key derivation from passwords with salt generation
- **Device Fingerprinting**: Unique device identification for enhanced security
- **Secure Storage**: OS keyring integration for Native platforms

#### 🌍 Cross-Platform Implementation

- **WASM-specific**: BroadcastChannel for cross-tab sync, localStorage/IndexedDB for persistence
- **Native-specific**: File-based IPC, filesystem storage, OS keyring integration
- **Platform Detection**: Automatic platform-aware feature selection

### ⚡ Performance Improvements

- **Efficient Lock Management**: Parking_lot for high-performance synchronization
- **Memory Optimization**: Smart session cleanup and expiry management
- **Async Architecture**: Full async/await support throughout session management
- **Background Tasks**: Automatic session cleanup and monitoring

### 🧪 Testing & Quality

- **54 Unit Tests**: Comprehensive test coverage for all modules
- **Integration Examples**: Production-ready session management example
- **Cross-platform Testing**: WASM and Native platform validation
- **Memory Safety**: Zero unsafe code with Rust's ownership guarantees

### 🔧 Developer Experience

- **Type Safety**: Full compile-time type checking for all session operations
- **Feature Flags**: Granular feature control (`session-management`, `session-encryption`, etc.)
- **Rich Documentation**: Comprehensive rustdoc with examples
- **Error Handling**: Detailed error contexts with platform-specific information

### 🔄 Breaking Changes

- **New Dependencies**: Added optional dependencies for session management features
- **Feature Flags**: Session management functionality requires `session-management` feature
- **Error Types**: New error variants (`Platform`, `Crypto`) for session management

### 📚 Documentation

- **Updated Examples**: New session management example demonstrating all features
- **Platform Guides**: Documentation for WASM vs Native differences
- **Security Guidelines**: Best practices for session encryption and key management

## [0.3.2] - 2025-08-15

### 🚀 Major Features Added

#### 🔐 Multi-Factor Authentication (MFA)

- **TOTP Support**: Time-based One-Time Password authentication compatible with Google Authenticator, Authy, and other TOTP apps
- **SMS 2FA**: SMS-based two-factor authentication with international phone number support
- **QR Code Generation**: Automatic QR code generation for TOTP setup with console-friendly ASCII output
- **Factor Management**: Complete MFA factor lifecycle management (list, create, verify, delete)
- **Challenge Flow**: Full MFA challenge creation and verification workflow

#### 🔄 Advanced OAuth Token Management

- **Advanced Token Refresh**: Intelligent token refresh with comprehensive error handling and retry logic
- **Token Metadata**: Detailed token information including expiry tracking, refresh counts, and scope information
- **Local Token Validation**: Client-side token validation without API calls for improved performance
- **Buffer-based Refresh**: Configurable refresh buffers to prevent token expiry
- **Enhanced Error Recovery**: Platform-specific error context with intelligent retry suggestions

#### 📱 Enhanced Phone Number Processing

- **International Support**: Comprehensive international phone number validation with country codes
- **Phone Formatting**: Automatic phone number formatting in international format
- **Validation Pipeline**: Multi-stage phone number validation with detailed error reporting

### ✨ API Additions

#### New MFA Methods

```rust
// Setup TOTP authentication
let totp_setup = client.auth()
    .setup_totp("My Authenticator")
    .await?;
println!("QR Code: {}", totp_setup.qr_code);

// Setup SMS MFA with international number
let factor = client.auth()
    .setup_sms_mfa("+1-555-123-4567", "My Phone", Some("US"))
    .await?;

// Create and verify MFA challenge
let challenge = client.auth().create_mfa_challenge(factor_id).await?;
let result = client.auth()
    .verify_mfa_challenge(factor_id, challenge.id, "123456")
    .await?;

// List and manage MFA factors
let factors = client.auth().list_mfa_factors().await?;
client.auth().delete_mfa_factor(factor_id).await?;
```

#### New Token Management Methods

```rust
// Advanced token refresh with error handling
match client.auth().refresh_token_advanced().await {
    Ok(session) => println!("Token refreshed successfully!"),
    Err(e) if e.is_retryable() => {
        println!("Retry after {} seconds", e.retry_after().unwrap_or(60));
    }
    Err(e) => println!("Re-authentication required: {}", e),
}

// Token metadata and validation
let metadata = client.auth().get_token_metadata()?;
let is_valid = client.auth().validate_token_local()?;
let time_left = client.auth().time_until_expiry()?;

// Smart refresh with configurable buffer
if client.auth().needs_refresh_with_buffer(300)? {
    client.auth().refresh_token_advanced().await?;
}
```

#### Enhanced Error Context

```rust
// Rich error information
if let Some(context) = error.context() {
    println!("Platform: {:?}", context.platform);
    if let Some(http) = &context.http {
        println!("Status: {:?}", http.status_code);
    }
    if let Some(retry) = &context.retry {
        println!("Retryable: {}", retry.retryable);
    }
}
```

### 🔧 Dependencies Added

- **totp-rs**: `5.6` - TOTP generation and validation
- **base32**: `0.5` - Base32 encoding/decoding for TOTP secrets
- **qrcode**: `0.14` - QR code generation for TOTP setup
- **image**: `0.25` - Image processing support for QR codes
- **phonenumber**: `0.3` - International phone number validation and formatting
- **dirs**: `5.0` - Cross-platform directory paths for session storage

### 📊 Performance & Quality

- **36 Comprehensive Tests**: All new MFA and token management functionality fully tested
- **100% Test Success Rate**: All tests passing with comprehensive coverage
- **Enhanced Documentation**: Complete rustdoc documentation with working examples
- **Cross-Platform Compatibility**: Full Native and WASM support maintained
- **Type Safety**: All new APIs are fully type-safe with comprehensive error handling

### 🐛 Bug Fixes

- **Error Context**: Fixed error context initialization with proper platform detection
- **Phone Validation**: Improved phone number parsing with better error messages
- **Token Refresh**: Enhanced token refresh logic with proper session management
- **Memory Management**: Optimized memory usage for MFA factor storage

### 📚 Documentation Improvements

- **MFA Guide**: Comprehensive MFA setup and usage examples
- **Token Management**: Detailed token management strategies and best practices
- **Error Handling**: Enhanced error handling patterns with retry logic examples
- **Phone Authentication**: International phone authentication patterns

### 🌟 Breaking Changes

- **None**: v0.3.2 is fully backward compatible with v0.3.1

### 🔜 Next Steps (v0.4.0)

- Full Cross-Platform & Advanced Features
- React Native and Node.js compatibility improvements
- Multi-language bindings (Python, Go, C#)
- Advanced caching and offline support

---

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
