//! Edge Functions example for Supabase client
//!
//! This example demonstrates how to invoke Supabase Edge Functions.
//! It shows basic function calls, passing parameters, and handling responses.
//!
//! To run this example:
//! ```bash
//! cargo run --example functions_example --features functions
//! ```

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct ProcessResult {
    success: bool,
    message: String,
    data: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for better logging
    tracing_subscriber::fmt::init();

    println!("🚀 Edge Functions Example");
    println!("=========================\n");

    // Initialize client
    let client = supabase::Client::new(
        "http://localhost:54321",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
    )?;

    // Example 1: Simple function call
    println!("⚡ Example 1: Simple function call");
    match client.functions().invoke("hello-world", None).await {
        Ok(result) => println!("✅ Function result: {}", result),
        Err(e) => println!("⚠️ Function call failed (expected): {}", e),
    }

    // Example 2: Function with parameters
    println!("\n⚡ Example 2: Function with parameters");
    let params = json!({
        "name": "Rust Developer",
        "message": "Hello from Supabase Rust Client!",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    match client.functions().invoke("greet-user", Some(params)).await {
        Ok(result) => {
            println!("✅ Function result: {}", result);

            // Try to parse specific result format
            if let Ok(parsed) = serde_json::from_value::<ProcessResult>(result) {
                println!("   📋 Parsed result:");
                println!("      Success: {}", parsed.success);
                println!("      Message: {}", parsed.message);
                if let Some(data) = parsed.data {
                    println!("      Data: {}", data);
                }
            }
        }
        Err(e) => println!("⚠️ Function call failed (expected): {}", e),
    }

    // Example 3: Function with custom headers
    println!("\n⚡ Example 3: Function with custom headers");
    let mut headers = HashMap::new();
    headers.insert("X-API-Version".to_string(), "v1".to_string());
    headers.insert("X-Client".to_string(), "supabase-rust".to_string());
    headers.insert("X-Request-ID".to_string(), uuid::Uuid::new_v4().to_string());

    let auth_data = json!({
        "user_id": "12345",
        "action": "authenticate",
        "permissions": ["read", "write"]
    });

    match client
        .functions()
        .invoke_with_options("secure-function", Some(auth_data), Some(headers))
        .await
    {
        Ok(result) => println!("✅ Secure function result: {}", result),
        Err(e) => println!("⚠️ Secure function failed (expected): {}", e),
    }

    // Example 4: Processing data function
    println!("\n⚡ Example 4: Data processing function");
    let processing_data = json!({
        "operation": "transform",
        "data": [
            {"id": 1, "name": "Item 1", "value": 100},
            {"id": 2, "name": "Item 2", "value": 200},
            {"id": 3, "name": "Item 3", "value": 300}
        ],
        "filters": {
            "min_value": 150,
            "include_metadata": true
        }
    });

    match client
        .functions()
        .invoke("process-data", Some(processing_data))
        .await
    {
        Ok(result) => {
            println!("✅ Data processing result: {}", result);

            // Example of handling array responses
            if let Some(array) = result.as_array() {
                println!("   📊 Processed {} items", array.len());
                for (i, item) in array.iter().enumerate() {
                    println!("      Item {}: {}", i + 1, item);
                }
            }
        }
        Err(e) => println!("⚠️ Data processing failed (expected): {}", e),
    }

    // Example 5: Error handling
    println!("\n⚡ Example 5: Error handling");
    match client
        .functions()
        .invoke("non-existent-function", None)
        .await
    {
        Ok(result) => println!("✅ Unexpected success: {}", result),
        Err(e) => {
            println!("⚠️ Expected error occurred: {}", e);
            println!("   This demonstrates proper error handling for missing functions");
        }
    }

    // Example 6: Async workflow function
    println!("\n⚡ Example 6: Async workflow function");
    let workflow_data = json!({
        "workflow_id": uuid::Uuid::new_v4(),
        "steps": [
            {"name": "validate", "timeout": 5},
            {"name": "process", "timeout": 30},
            {"name": "notify", "timeout": 10}
        ],
        "callback_url": "https://my-app.com/webhook"
    });

    match client
        .functions()
        .invoke("start-workflow", Some(workflow_data))
        .await
    {
        Ok(result) => {
            println!("✅ Workflow started: {}", result);

            // Extract workflow ID from response
            if let Some(workflow_id) = result.get("workflow_id") {
                println!("   🔄 Workflow ID: {}", workflow_id);
            }

            if let Some(status) = result.get("status") {
                println!("   📋 Status: {}", status);
            }
        }
        Err(e) => println!("⚠️ Workflow start failed (expected): {}", e),
    }

    println!("\n🎉 Edge Functions example completed!");
    println!("\n💡 Tips:");
    println!("   - Functions run on the edge for low latency");
    println!("   - Use custom headers for authentication or API versioning");
    println!("   - Handle errors gracefully as functions may be unavailable");
    println!("   - Functions are ideal for serverless data processing");
    println!("   - Local development requires Supabase CLI: 'supabase functions serve'");

    Ok(())
}
