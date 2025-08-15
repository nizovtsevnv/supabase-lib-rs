# Architecture

This document describes the architecture and design principles of the Supabase Rust Client library.

## Overview

The Supabase Rust Client is designed as a modular, type-safe, and cross-platform library that provides comprehensive access to all Supabase services. The architecture prioritizes:

- **Type Safety**: Full Rust type system integration
- **Performance**: Efficient async operations with minimal overhead
- **Modularity**: Clean separation of concerns between services
- **Cross-Platform**: Native and WebAssembly support
- **Extensibility**: Plugin-friendly architecture

## Core Architecture

### Client Structure

```
Client (Central Hub)
├── Auth Module        - Authentication & session management
├── Database Module    - PostgREST operations & raw SQL
├── Storage Module     - File operations & bucket management
├── Functions Module   - Edge function invocation
├── Realtime Module    - WebSocket subscriptions & presence
└── HTTP Client        - Shared request/response handling
```

### Module Design

Each module follows a consistent pattern:

1. **Service Interface** - High-level API for common operations
2. **Builder Pattern** - Fluent APIs for complex operations  
3. **Type Safety** - Compile-time guarantees for correctness
4. **Error Handling** - Comprehensive error types and context
5. **Async/Await** - Full async support with proper cancellation

### Cross-Platform Support

The library supports two main targets:

#### Native (Tokio Runtime)
- Uses `reqwest` for HTTP operations
- Native WebSocket support via `tokio-tungstenite`
- File system access for session storage
- Full feature set available

#### WebAssembly (WASM)
- Uses `web-sys` for browser APIs
- WebSocket through browser's WebSocket API
- LocalStorage for session persistence
- Optimized for bundle size

## Service Modules

### Authentication Module

**Responsibility**: User authentication, session management, and security

**Key Components**:
- `Auth` - Main authentication interface
- `Session` - User session representation
- `User` - User profile and metadata
- `SessionManager` - Cross-platform session storage
- `MfaChallenge` - Multi-factor authentication

**Design Patterns**:
- Session-based state management
- Event-driven authentication state changes
- Secure token handling with automatic refresh
- Cross-tab session synchronization

### Database Module

**Responsibility**: Database operations via PostgREST API

**Key Components**:
- `Database` - Main database interface
- `QueryBuilder` - Fluent query construction
- `Filter` - Type-safe filtering operations
- `Transaction` - Database transaction support
- `RawSql` - Direct SQL execution

**Design Patterns**:
- Builder pattern for query construction
- Type-safe filters and joins
- Prepared statement support
- Connection pooling for performance

### Storage Module

**Responsibility**: File storage and bucket management

**Key Components**:
- `Storage` - Main storage interface
- `Bucket` - Bucket management operations
- `FileMetadata` - File information and metadata
- `UploadSession` - Resumable upload support
- `StoragePolicy` - Row-level security helpers

**Design Patterns**:
- Streaming upload/download for large files
- Progress callback support
- Automatic retry with exponential backoff
- Policy-based access control

### Functions Module

**Responsibility**: Serverless edge function invocation

**Key Components**:
- `Functions` - Main functions interface
- `InvokeOptions` - Function invocation configuration
- `FunctionMetadata` - Function information
- `StreamingResponse` - Streaming function responses

**Design Patterns**:
- Request/response streaming
- Timeout and retry handling
- Function metadata caching
- Local testing support

### Realtime Module

**Responsibility**: Real-time subscriptions and presence

**Key Components**:
- `Realtime` - Main realtime interface
- `RealtimeChannel` - Channel subscription management
- `Presence` - User presence tracking
- `Broadcast` - Message broadcasting
- `WebSocketManager` - Connection management

**Design Patterns**:
- Event-driven message handling
- Automatic reconnection with backoff
- Message queuing during disconnection
- Presence state synchronization

## Error Handling Strategy

### Error Hierarchy

```
SupabaseError (Root Error)
├── AuthError - Authentication failures
├── DatabaseError - Database operation errors
├── StorageError - File operation errors
├── FunctionsError - Edge function errors
├── RealtimeError - Real-time connection errors
└── ClientError - HTTP client and configuration errors
```

### Error Context

Each error includes:
- **Source** - Original error cause
- **Context** - Operation being performed
- **Retry Info** - Whether operation is retryable
- **Suggestions** - Potential solutions

## Performance Considerations

### Connection Management

- **HTTP Connection Pool**: Reused connections for efficiency
- **WebSocket Management**: Single connection per realtime instance
- **Request Batching**: Automatic batching for bulk operations
- **Caching Strategy**: Intelligent caching for metadata and tokens

### Memory Management

- **Zero-Copy Operations**: Where possible, avoid unnecessary allocations
- **Streaming**: Large file operations use streaming to minimize memory usage
- **Resource Cleanup**: Automatic cleanup of connections and resources
- **WASM Optimization**: Minimized bundle size for web deployment

## Security Architecture

### Token Management

- **Automatic Refresh**: Tokens refreshed before expiration
- **Secure Storage**: Platform-appropriate secure storage
- **Token Validation**: Local token validation for performance
- **Session Encryption**: Optional session encryption for sensitive data

### Network Security

- **TLS/SSL**: All communications encrypted in transit
- **Certificate Validation**: Proper certificate chain validation
- **Request Signing**: HMAC signing for sensitive operations
- **Rate Limiting**: Client-side rate limiting respect

## Testing Strategy

### Test Categories

- **Unit Tests** (41 tests): Individual component testing
- **Integration Tests**: Service integration testing
- **End-to-End Tests**: Full workflow testing
- **Documentation Tests** (72 tests): Example code validation
- **Performance Tests**: Benchmarking critical paths

### Test Environment

- **Docker Compose**: Local Supabase stack for testing
- **Test Fixtures**: Shared test data and utilities
- **Mock Services**: Isolated component testing
- **CI/CD Integration**: Automated testing on all platforms

## Extensibility

### Plugin Architecture

The library is designed to support extensions:

- **Custom Auth Providers**: Additional authentication methods
- **Storage Backends**: Alternative storage implementations
- **Database Adapters**: Support for additional databases
- **Middleware Support**: Request/response transformation

### Configuration System

- **Environment Variables**: Standard configuration
- **Builder Pattern**: Programmatic configuration
- **Feature Flags**: Conditional compilation
- **Runtime Configuration**: Dynamic behavior modification

## Future Considerations

### Planned Improvements

- **Connection Multiplexing**: HTTP/2 and HTTP/3 support
- **Advanced Caching**: More sophisticated caching strategies
- **Performance Monitoring**: Built-in performance metrics
- **Enhanced Security**: Additional security features

### Compatibility

- **Rust Editions**: Support for current and future Rust editions
- **Supabase API**: Backward compatibility with Supabase API versions
- **Platform Support**: Expanded platform and architecture support
- **MSRV Policy**: Minimum Supported Rust Version maintenance

## Related Documentation

- [Contributing Guide](../CONTRIBUTING.md)
- [Testing Guide](../TESTING.md)
- [WebAssembly Guide](WASM_GUIDE.md)
- [Configuration Guide](CONFIGURATION.md)
- [API Documentation](https://docs.rs/supabase-lib-rs) 