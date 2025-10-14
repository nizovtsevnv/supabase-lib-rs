//! WASM example for Supabase client
//!
//! This example demonstrates how to use the Supabase client in a WASM environment.
//! It compiles to WebAssembly and can be used in web browsers with Dioxus or other WASM frameworks.
//!
//! To compile for WASM:
//! ```bash
//! cargo build --target wasm32-unknown-unknown --example wasm_example
//! ```

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    id: Option<i32>,
    title: String,
    completed: bool,
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! console_log {
    ($($t:tt)*) => (println!($($t)*))
}

#[cfg(target_arch = "wasm32")]
async fn run_supabase_example() {
    console_log!("🚀 Starting Supabase WASM example");

    // Initialize Supabase client
    let client = match supabase::Client::new(
        "http://localhost:54321",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
    ) {
        Ok(client) => {
            console_log!("✅ Supabase client initialized");
            client
        }
        Err(e) => {
            console_log!("❌ Failed to create client: {}", e);
            return;
        }
    };

    // Test authentication
    console_log!("🔐 Testing authentication...");
    let auth_result = client
        .auth()
        .sign_in_with_email_and_password("test@example.com", "password123")
        .await;

    match auth_result {
        Ok(response) => {
            if let Some(user) = response.user {
                console_log!(
                    "✅ Authentication successful: {}",
                    user.email.unwrap_or_default()
                );
            }
        }
        Err(e) => console_log!("⚠️ Auth failed (expected in demo): {}", e),
    }

    // Test database operations
    console_log!("🗄️ Testing database operations...");

    // Try to fetch data
    let query_result = client
        .database()
        .from("todos")
        .select("*")
        .limit(5)
        .execute::<Todo>()
        .await;

    match query_result {
        Ok(todos) => {
            console_log!("✅ Database query successful: {} todos found", todos.len());
            for todo in todos {
                console_log!("  - {}: {}", todo.id.unwrap_or(0), todo.title);
            }
        }
        Err(e) => console_log!("⚠️ Database query failed (expected without data): {}", e),
    }

    // Test storage operations
    #[cfg(feature = "storage")]
    {
        console_log!("📁 Testing storage operations...");
        let storage_result = client.storage().list_buckets().await;

        match storage_result {
            Ok(buckets) => console_log!("✅ Storage check successful: {} buckets", buckets.len()),
            Err(e) => console_log!("⚠️ Storage check failed: {}", e),
        }
    }

    // Test realtime connection
    #[cfg(feature = "realtime")]
    {
        console_log!("⚡ Testing realtime connection...");
        let realtime = client.realtime();

        match realtime.connect().await {
            Ok(_) => {
                console_log!("✅ Realtime connected successfully");
                let _channel = realtime.channel("test-channel");
                console_log!("✅ Channel created: test-channel");

                // Disconnect
                if let Err(e) = realtime.disconnect().await {
                    console_log!("⚠️ Disconnect failed: {}", e);
                } else {
                    console_log!("✅ Realtime disconnected");
                }
            }
            Err(e) => console_log!("⚠️ Realtime connection failed: {}", e),
        }
    }

    console_log!("🎉 WASM example completed successfully!");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("🌐 Running Supabase client in WASM environment");

    wasm_bindgen_futures::spawn_local(async {
        run_supabase_example().await;
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    console_log!("🖥️  This example is designed for WASM. Use 'cargo build --target wasm32-unknown-unknown --example wasm_example'");
}
