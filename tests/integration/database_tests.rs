//! Integration tests for database module

use supabase_rs::prelude::*;
use serde_json::json;

mod common;
use common::*;

#[tokio::test]
async fn test_database_module_initialization() {
    let client = create_test_client();
    let database = client.database();

    // Should be able to create query builders
    let _query = database.from("test_table");
    let _insert = database.insert("test_table");
    let _update = database.update("test_table");
    let _delete = database.delete("test_table");
}

#[tokio::test]
async fn test_query_builder_construction() {
    let client = create_test_client();

    let query = client.database()
        .from("users")
        .select("id, name, email")
        .eq("active", "true")
        .gt("age", "18")
        .limit(10)
        .offset(5);

    // Query builder should be constructible without errors
    // The actual execution would require a real database
}

#[tokio::test]
async fn test_insert_builder_construction() {
    let client = create_test_client();

    let test_data = TestRecord::new("Test User")
        .with_description("A test user record");

    let insert_result = client.database()
        .insert("users")
        .values(test_data);

    assert!(insert_result.is_ok());

    let insert = insert_result.unwrap()
        .upsert()
        .returning("*");

    // Insert builder should be constructible
}

#[tokio::test]
async fn test_update_builder_construction() {
    let client = create_test_client();

    let update_data = json!({
        "name": "Updated Name",
        "description": "Updated description"
    });

    let update_result = client.database()
        .update("users")
        .set(update_data);

    assert!(update_result.is_ok());

    let update = update_result.unwrap()
        .eq("id", "123")
        .returning("*");

    // Update builder should be constructible
}

#[tokio::test]
async fn test_delete_builder_construction() {
    let client = create_test_client();

    let delete = client.database()
        .delete("users")
        .eq("id", "123")
        .returning("id, name");

    // Delete builder should be constructible
}

async_test!(test_rpc_call, || async {
    let client = create_test_client();

    let params = json!({
        "param1": "value1",
        "param2": 42
    });

    let result = client.database()
        .rpc("test_function", Some(params))
        .await;

    // In test environment, RPC might fail due to function not existing
    match result {
        Ok(response) => {
            println!("RPC call succeeded: {:?}", response);
        },
        Err(e) => {
            println!("RPC call failed as expected: {}", e);
            // Should contain error about missing function or connection
        }
    }
});

async_test!(test_query_execution_mock, || async {
    let client = create_test_client();

    // This would fail in a real environment without proper setup
    let result = client.database()
        .from("non_existent_table")
        .select("*")
        .execute::<TestRecord>()
        .await;

    // Should return an error about table not existing or connection issues
    assert!(result.is_err());
    let error = result.unwrap_err();
    println!("Expected error: {}", error);
});

async_test!(test_single_query_execution, || async {
    let client = create_test_client();

    let result = client.database()
        .from("test_table")
        .select("*")
        .eq("id", "1")
        .single_execute::<TestRecord>()
        .await;

    // Should fail in test environment
    assert!(result.is_err());
});

async_test!(test_insert_execution, || async {
    let client = create_test_client();

    let test_record = TestRecord::new("Test Insert");

    let result = client.database()
        .insert("test_table")
        .values(test_record)
        .unwrap()
        .execute::<TestRecord>()
        .await;

    // Should fail in test environment without proper database setup
    assert!(result.is_err());
});

async_test!(test_update_execution, || async {
    let client = create_test_client();

    let update_data = json!({
        "name": "Updated Name"
    });

    let result = client.database()
        .update("test_table")
        .set(update_data)
        .unwrap()
        .eq("id", "1")
        .execute::<TestRecord>()
        .await;

    // Should fail in test environment
    assert!(result.is_err());
});

async_test!(test_delete_execution, || async {
    let client = create_test_client();

    let result = client.database()
        .delete("test_table")
        .eq("id", "1")
        .execute::<TestRecord>()
        .await;

    // Should fail in test environment
    assert!(result.is_err());
});

#[tokio::test]
async fn test_query_builder_chaining() {
    let client = create_test_client();

    // Test method chaining
    let query = client.database()
        .from("posts")
        .select("id, title, content, author_id")
        .eq("published", "true")
        .gt("created_at", "2023-01-01")
        .like("title", "%rust%")
        .order("created_at", supabase_rs::types::OrderDirection::Descending)
        .limit(20);

    // Chaining should work without errors
}

#[tokio::test]
async fn test_filter_operators() {
    let client = create_test_client();

    // Test all filter operators
    let _query = client.database()
        .from("test")
        .eq("field1", "value1")
        .neq("field2", "value2")
        .gt("field3", "100")
        .gte("field4", "100")
        .lt("field5", "200")
        .lte("field6", "200")
        .like("field7", "%pattern%")
        .ilike("field8", "%PATTERN%")
        .is("field9", "null")
        .r#in("field10", &["val1", "val2", "val3"]);

    // All operators should be available
}

// Note: Full integration tests would require:
// - Real Supabase database with test tables
// - Proper authentication setup
// - Test data fixtures
// - Cleanup after tests
// - Performance testing for large datasets
// - Transaction testing
// - Concurrent operation testing
