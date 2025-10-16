//! Edge Functions & Performance example for Supabase client (v0.4.2)
//!
//! This example demonstrates the new v0.4.2 features:
//! - Enhanced Edge Functions with metadata, retry logic, and local testing
//! - Performance optimization with connection pooling and caching
//! - Streaming responses and batch operations
//!
//! To run this example:
//! ```bash
//! cargo run --example functions_performance_example --features "functions,performance"
//! ```

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, time::Duration};

#[cfg(feature = "functions")]
use supabase_lib_rs::functions::{InvokeOptions, LocalConfig, RetryConfig};

#[cfg(feature = "performance")]
use supabase_lib_rs::performance::{BatchOperation, CacheConfig, ConnectionPoolConfig};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct FunctionResult {
    success: bool,
    message: String,
    data: Option<serde_json::Value>,
    timestamp: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    println!("🚀 Edge Functions & Performance Demo (v0.4.2)");
    println!("===============================================\n");

    // Initialize client
    let client = supabase_lib_rs::Client::new(
        "http://localhost:54321",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
    )?;

    // Demo 1: Enhanced Edge Functions
    #[cfg(feature = "functions")]
    {
        println!("⚡ 1. Enhanced Edge Functions");
        println!("──────────────────────────────\n");

        // Example 1.1: Function Metadata
        println!("📊 Function Metadata:");
        match client.functions().list_functions().await {
            Ok(functions) => {
                if functions.is_empty() {
                    println!("   ℹ️  No functions found (this is expected for demo)");
                } else {
                    for func in functions {
                        println!("   📄 Function: {} - Status: {:?}", func.name, func.status);
                        if let Some(desc) = func.description {
                            println!("      Description: {}", desc);
                        }
                        if let Some(memory) = func.memory_limit {
                            println!("      Memory: {}MB", memory);
                        }
                    }
                }
            }
            Err(e) => println!("   ⚠️  Metadata fetch failed (expected): {}", e),
        }
        println!();

        // Example 1.2: Advanced Function Invocation with Retry
        println!("🔄 Advanced Invocation with Retry:");
        let mut headers = HashMap::new();
        headers.insert("X-Priority".to_string(), "high".to_string());
        headers.insert("X-Custom-Metadata".to_string(), "demo-v042".to_string());

        let options = InvokeOptions {
            headers: Some(headers),
            timeout: Some(Duration::from_secs(10)),
            retry: Some(RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(500),
                backoff_multiplier: 2.0,
                max_delay: Duration::from_secs(5),
            }),
            streaming: false,
        };

        let payload = json!({
            "message": "Hello from v0.4.2!",
            "features": ["metadata", "retry", "local-testing", "streaming"],
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        match client
            .functions()
            .invoke_with_advanced_options("demo-function", Some(payload), options)
            .await
        {
            Ok(result) => println!("   ✅ Function result: {}", result),
            Err(e) => println!("   ⚠️  Function call failed (expected): {}", e),
        }
        println!();

        // Example 1.3: Local Development Testing
        println!("🧪 Local Development Testing:");
        let local_config = LocalConfig {
            local_url: "http://localhost:54321".to_string(),
            functions_dir: Some("./functions".to_string()),
            port: Some(54321),
        };

        match client
            .functions()
            .test_local(
                "test-function",
                Some(json!({"test": true, "env": "local"})),
                local_config,
            )
            .await
        {
            Ok(result) => println!("   ✅ Local test result: {}", result),
            Err(e) => println!("   ℹ️  Local test failed (expected): {}", e),
        }
        println!();

        // Example 1.4: Streaming Functions (native only)
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("📡 Streaming Functions:");
            use tokio_stream::StreamExt;

            match client
                .functions()
                .invoke_stream(
                    "streaming-demo",
                    Some(json!({"stream_count": 3, "delay_ms": 1000})),
                )
                .await
            {
                Ok(mut stream) => {
                    println!("   🔄 Streaming started...");
                    let mut chunk_count = 0;
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                chunk_count += 1;
                                if chunk.is_final {
                                    println!("   🏁 Stream completed after {} chunks", chunk_count);
                                    break;
                                } else {
                                    println!("   📦 Chunk {}: {:?}", chunk_count, chunk.data);
                                }
                            }
                            Err(e) => {
                                println!("   ❌ Stream error: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => println!("   ⚠️  Streaming failed (expected): {}", e),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            println!("   ℹ️  Streaming functions not available on WASM");
        }
        println!();
    }

    #[cfg(not(feature = "functions"))]
    {
        println!("⚠️  Edge Functions features disabled");
        println!("    💡 Run with: --features functions");
        println!();
    }

    // Demo 2: Performance Optimization
    #[cfg(feature = "performance")]
    {
        println!("🚀 2. Performance Optimization");
        println!("──────────────────────────────\n");

        // Initialize performance manager
        let pool_config = ConnectionPoolConfig {
            max_connections_per_host: 20,
            idle_timeout: Duration::from_secs(120),
            keep_alive_timeout: Duration::from_secs(90),
            http2: true,
            user_agent: Some("supabase-rust-demo/0.4.2".to_string()),
        };

        let cache_config = CacheConfig {
            max_entries: 500,
            default_ttl: Duration::from_secs(600),
            enable_compression: true,
            cache_success_only: true,
        };

        // Example 2.1: Connection Pooling
        println!("🔗 Connection Pooling Demo:");
        println!(
            "   📊 Pool config: {} max connections per host",
            pool_config.max_connections_per_host
        );
        println!("   ⏱️  Idle timeout: {:?}", pool_config.idle_timeout);
        println!("   🔄 Keep-alive: {:?}", pool_config.keep_alive_timeout);
        println!();

        // Example 2.2: Request Caching
        println!("💾 Request Caching Demo:");
        println!(
            "   📈 Cache config: {} max entries",
            cache_config.max_entries
        );
        println!("   ⏰ Default TTL: {:?}", cache_config.default_ttl);
        println!("   🗜️  Compression: {}", cache_config.enable_compression);

        // Simulate cache usage
        let test_data = json!({
            "cached_data": "This is cached response",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": "0.4.2"
        });
        println!("   💽 Cached example data: {}", test_data);
        println!();

        // Example 2.3: Batch Operations
        println!("📦 Batch Operations Demo:");
        let batch_operations = vec![
            BatchOperation {
                id: "op_1".to_string(),
                method: "GET".to_string(),
                url: "http://localhost:54321/rest/v1/users".to_string(),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("apikey".to_string(), "demo-key".to_string());
                    h
                },
                body: None,
                priority: 1,
            },
            BatchOperation {
                id: "op_2".to_string(),
                method: "POST".to_string(),
                url: "http://localhost:54321/rest/v1/logs".to_string(),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("apikey".to_string(), "demo-key".to_string());
                    h.insert("Content-Type".to_string(), "application/json".to_string());
                    h
                },
                body: Some(json!({"action": "demo", "timestamp": chrono::Utc::now()})),
                priority: 2,
            },
        ];

        println!(
            "   📊 Prepared {} batch operations:",
            batch_operations.len()
        );
        for op in &batch_operations {
            println!(
                "     - {} {}: {} (priority: {})",
                op.method, op.id, op.url, op.priority
            );
        }
        println!();

        // Example 2.4: Performance Metrics
        println!("📈 Performance Metrics:");
        println!("   🔢 Total operations: {}", batch_operations.len());
        println!("   ⚡ Average response time: ~45ms (simulated)");
        println!("   📊 Cache hit ratio: 85% (simulated)");
        println!("   🔗 Active connections: 3 (simulated)");
        println!();
    }

    #[cfg(not(feature = "performance"))]
    {
        println!("⚠️  Performance features disabled");
        println!("    💡 Run with: --features performance");
        println!();
    }

    // Demo 3: Integration Example
    println!("🔧 3. Integration Showcase");
    println!("──────────────────────────────\n");

    println!("🎯 v0.4.2 Key Features Demonstrated:");
    println!("   ⚡ Enhanced Edge Functions:");
    println!("      - Function metadata and introspection");
    println!("      - Advanced retry mechanisms with exponential backoff");
    println!("      - Local development and testing utilities");
    println!("      - Streaming responses for real-time data");
    println!("      - Enhanced error handling and context");
    println!();

    println!("   🚀 Performance Optimization:");
    println!("      - HTTP connection pooling and management");
    println!("      - Intelligent request/response caching");
    println!("      - Batch operation processing");
    println!("      - Performance metrics and monitoring");
    println!("      - Request/response compression support");
    println!();

    println!("✨ Production-Ready Features:");
    println!("   🔒 Security: Enhanced error handling without data leaks");
    println!("   📊 Monitoring: Built-in performance metrics");
    println!("   🔄 Resilience: Retry logic with exponential backoff");
    println!("   ⚡ Performance: Connection pooling and caching");
    println!("   🧪 Development: Local testing and debugging utilities");
    println!();

    println!("🎉 Demo completed! v0.4.2 adds significant performance and developer experience improvements.");

    Ok(())
}
