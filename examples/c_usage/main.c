#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../../include/supabase.h"

int main() {
    printf("=== Supabase C FFI Example ===\n\n");

    // Create client
    SupabaseClient* client = supabase_client_new(
        "http://localhost:54321",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
    );

    if (client == NULL) {
        printf("❌ Failed to create Supabase client\n");
        return 1;
    }

    printf("✅ Supabase client created successfully\n");

    // Authentication example
    char auth_result[1024];
    SupabaseError error = supabase_auth_sign_in(
        client,
        "test@example.com",
        "password123",
        auth_result,
        sizeof(auth_result)
    );

    if (error == SUPABASE_SUCCESS) {
        printf("✅ Auth response: %s\n", auth_result);
    } else {
        printf("❌ Auth failed with error code: %d\n", error);
    }

    // Database query example
    char db_result[2048];
    error = supabase_database_select(
        client,
        "users",
        "*",
        db_result,
        sizeof(db_result)
    );

    if (error == SUPABASE_SUCCESS) {
        printf("✅ Database query result: %s\n", db_result);
    } else {
        printf("❌ Database query failed with error code: %d\n", error);
    }

    // Cleanup
    supabase_client_free(client);
    printf("✅ Client cleaned up\n");

    printf("\n🎉 C FFI example completed!\n");
    return 0;
}
