#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../../include/supabase.h"

void print_error(const char* operation) {
    char error_buf[512];
    SupabaseError error = supabase_get_last_error(error_buf, sizeof(error_buf));
    if (error == SUPABASE_SUCCESS) {
        printf("❌ %s failed: %s\n", operation, error_buf);
    } else {
        printf("❌ %s failed with unknown error\n", operation);
    }
}

int main() {
    printf("=== Enhanced Supabase C FFI Example ===\n\n");

    // Create client with real connection
    SupabaseClient* client = supabase_client_new(
        "http://localhost:54321",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
    );

    if (client == NULL) {
        printf("❌ Failed to create Supabase client\n");
        print_error("Client creation");
        return 1;
    }

    printf("✅ Supabase client created successfully\n");

    // Authentication examples
    printf("\n📋 Testing Authentication...\n");

    // Sign up new user
    char signup_result[2048];
    SupabaseError error = supabase_auth_sign_up(
        client,
        "testuser@example.com",
        "securepassword123",
        signup_result,
        sizeof(signup_result)
    );

    if (error == SUPABASE_SUCCESS) {
        printf("✅ Sign up successful: %s\n", signup_result);
    } else {
        print_error("Sign up");
    }

    // Sign in user
    char signin_result[2048];
    error = supabase_auth_sign_in(
        client,
        "testuser@example.com",
        "securepassword123",
        signin_result,
        sizeof(signin_result)
    );

    if (error == SUPABASE_SUCCESS) {
        printf("✅ Sign in successful\n");
    } else {
        print_error("Sign in");
    }

    // Database operations
    printf("\n📊 Testing Database Operations...\n");

    // Select from a table
    char db_result[4096];
    error = supabase_database_select(
        client,
        "profiles",
        "id, email, created_at",
        db_result,
        sizeof(db_result)
    );

    if (error == SUPABASE_SUCCESS) {
        printf("✅ Database select successful: %s\n", db_result);
    } else {
        print_error("Database select");
    }

    // Insert new record
    char insert_result[2048];
    const char* json_data = "{\"name\":\"John Doe\",\"email\":\"john@example.com\"}";
    error = supabase_database_insert(
        client,
        "profiles",
        json_data,
        insert_result,
        sizeof(insert_result)
    );

    if (error == SUPABASE_SUCCESS) {
        printf("✅ Database insert successful: %s\n", insert_result);
    } else {
        print_error("Database insert");
    }

    // Storage operations
    printf("\n📁 Testing Storage Operations...\n");

    char storage_result[2048];
    error = supabase_storage_list_buckets(
        client,
        storage_result,
        sizeof(storage_result)
    );

    if (error == SUPABASE_SUCCESS) {
        printf("✅ Storage buckets listed: %s\n", storage_result);
    } else {
        print_error("Storage list buckets");
    }

    // Edge Functions
    printf("\n⚡ Testing Edge Functions...\n");

    char function_result[2048];
    const char* payload = "{\"message\":\"Hello from C!\"}";
    error = supabase_functions_invoke(
        client,
        "hello-world",
        payload,
        function_result,
        sizeof(function_result)
    );

    if (error == SUPABASE_SUCCESS) {
        printf("✅ Function invocation successful: %s\n", function_result);
    } else {
        print_error("Function invocation");
    }

    // Cleanup
    supabase_client_free(client);
    printf("\n✅ Client cleaned up successfully\n");

    printf("\n🎉 Enhanced C FFI example completed!\n");
    printf("📚 All major Supabase features tested through C FFI:\n");
    printf("   • Authentication (sign up, sign in)\n");
    printf("   • Database (select, insert)\n");
    printf("   • Storage (list buckets)\n");
    printf("   • Edge Functions (invoke)\n");
    printf("   • Comprehensive error handling\n");

    return 0;
}
