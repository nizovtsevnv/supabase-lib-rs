//! Integration tests for advanced Edge Functions features (v0.4.2)

use std::{collections::HashMap, time::Duration};
use supabase::functions::{
    FunctionMetadata, FunctionStatus, InvokeOptions, LocalConfig, RetryConfig, StreamChunk,
};

#[tokio::test]
async fn test_function_metadata_struct() {
    let metadata = FunctionMetadata {
        name: "test-function".to_string(),
        description: Some("Test function for v0.4.2".to_string()),
        version: Some("1.0.0".to_string()),
        runtime: Some("deno".to_string()),
        memory_limit: Some(256),
        timeout: Some(30),
        env_vars: {
            let mut env = HashMap::new();
            env.insert("NODE_ENV".to_string(), "test".to_string());
            env
        },
        status: FunctionStatus::Active,
        created_at: Some("2025-01-15T10:00:00Z".to_string()),
        updated_at: Some("2025-01-15T10:00:00Z".to_string()),
    };

    assert_eq!(metadata.name, "test-function");
    assert_eq!(metadata.memory_limit, Some(256));
    assert_eq!(metadata.status, FunctionStatus::Active);
    assert!(metadata.env_vars.contains_key("NODE_ENV"));
}

#[tokio::test]
async fn test_function_status_enum() {
    let active = FunctionStatus::Active;
    let inactive = FunctionStatus::Inactive;
    let deploying = FunctionStatus::Deploying;
    let failed = FunctionStatus::Failed;

    // Test serialization/deserialization
    let active_json = serde_json::to_string(&active).unwrap();
    let deserialized: FunctionStatus = serde_json::from_str(&active_json).unwrap();

    match deserialized {
        FunctionStatus::Active => {},
        _ => panic!("Expected Active status"),
    }
}

#[tokio::test]
async fn test_invoke_options_configuration() {
    let mut headers = HashMap::new();
    headers.insert("X-Test-Header".to_string(), "test-value".to_string());
    headers.insert("X-Priority".to_string(), "high".to_string());

    let retry_config = RetryConfig {
        max_attempts: 5,
        delay: Duration::from_millis(200),
        backoff_multiplier: 1.5,
        max_delay: Duration::from_secs(10),
    };

    let options = InvokeOptions {
        headers: Some(headers),
        timeout: Some(Duration::from_secs(15)),
        retry: Some(retry_config),
        streaming: false,
    };

    assert!(options.headers.is_some());
    assert_eq!(options.timeout, Some(Duration::from_secs(15)));
    assert!(options.retry.is_some());

    if let Some(retry) = &options.retry {
        assert_eq!(retry.max_attempts, 5);
        assert_eq!(retry.backoff_multiplier, 1.5);
    }
}

#[tokio::test]
async fn test_retry_config_defaults() {
    let default_config = RetryConfig::default();

    assert_eq!(default_config.max_attempts, 3);
    assert_eq!(default_config.delay, Duration::from_millis(1000));
    assert_eq!(default_config.backoff_multiplier, 2.0);
    assert_eq!(default_config.max_delay, Duration::from_secs(30));
}

#[tokio::test]
async fn test_stream_chunk_creation() {
    let chunk = StreamChunk {
        data: serde_json::json!({
            "message": "Hello streaming!",
            "chunk_id": 1
        }),
        sequence: Some(1),
        is_final: false,
    };

    assert!(chunk.data.is_object());
    assert_eq!(chunk.sequence, Some(1));
    assert!(!chunk.is_final);

    // Test final chunk
    let final_chunk = StreamChunk {
        data: serde_json::Value::Null,
        sequence: None,
        is_final: true,
    };

    assert!(final_chunk.is_final);
    assert!(final_chunk.data.is_null());
}

#[tokio::test]
async fn test_local_config_creation() {
    let local_config = LocalConfig {
        local_url: "http://localhost:54321".to_string(),
        functions_dir: Some("./supabase/functions".to_string()),
        port: Some(54321),
    };

    assert_eq!(local_config.local_url, "http://localhost:54321");
    assert_eq!(local_config.functions_dir, Some("./supabase/functions".to_string()));
    assert_eq!(local_config.port, Some(54321));
}

#[tokio::test]
async fn test_invoke_options_default() {
    let options = InvokeOptions::default();

    assert!(options.headers.is_none());
    assert!(options.timeout.is_none());
    assert!(options.retry.is_none());
    assert!(!options.streaming);
}

// Integration test with actual function calls would require a running Supabase instance
#[tokio::test]
#[ignore = "Requires running Supabase instance"]
async fn test_function_metadata_integration() {
    use crate::common::*;

    let client = create_test_client().await;

    // This test would require actual functions deployed
    match client.functions().list_functions().await {
        Ok(functions) => {
            println!("Found {} functions", functions.len());
            for func in functions {
                println!("Function: {} - Status: {:?}", func.name, func.status);
            }
        }
        Err(e) => {
            println!("Functions list failed: {}", e);
            // This is expected if no functions are deployed
        }
    }
}

// Stream processing test (native only)
#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_stream_processing_simulation() {
    use tokio_stream::{iter, StreamExt};

    // Simulate stream processing logic similar to functions.rs
    let lines = vec![
        "data: {\"message\": \"chunk 1\"}".to_string(),
        "data: {\"message\": \"chunk 2\"}".to_string(),
        "data: [DONE]".to_string(),
    ];

    let stream = iter(lines.into_iter().map(Ok::<String, std::io::Error>));
    let mut processed_chunks = Vec::new();

    let mut stream = stream.map(|line_result| {
        let line = line_result.unwrap();

        if line.starts_with("data: ") {
            let data_str = &line[6..];
            if data_str == "[DONE]" {
                return StreamChunk {
                    data: serde_json::Value::Null,
                    sequence: None,
                    is_final: true,
                };
            }

            let data: serde_json::Value = serde_json::from_str(data_str).unwrap_or(serde_json::Value::Null);
            StreamChunk {
                data,
                sequence: None,
                is_final: false,
            }
        } else {
            StreamChunk {
                data: serde_json::Value::Null,
                sequence: None,
                is_final: false,
            }
        }
    });

    while let Some(chunk) = stream.next().await {
        processed_chunks.push(chunk.clone());
        if chunk.is_final {
            break;
        }
    }

    assert_eq!(processed_chunks.len(), 3);
    assert!(processed_chunks[2].is_final);
    assert!(!processed_chunks[0].data.is_null());
}
