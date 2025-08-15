#ifndef SUPABASE_H
#define SUPABASE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Forward declarations
typedef struct SupabaseClient SupabaseClient;

// Enhanced error codes
typedef enum {
    SUPABASE_SUCCESS = 0,
    SUPABASE_INVALID_INPUT = 1,
    SUPABASE_NETWORK_ERROR = 2,
    SUPABASE_AUTH_ERROR = 3,
    SUPABASE_DATABASE_ERROR = 4,
    SUPABASE_STORAGE_ERROR = 5,
    SUPABASE_FUNCTIONS_ERROR = 6,
    SUPABASE_REALTIME_ERROR = 7,
    SUPABASE_RUNTIME_ERROR = 8,
    SUPABASE_UNKNOWN_ERROR = 99
} SupabaseError;

// Client management
SupabaseClient* supabase_client_new(const char* url, const char* key);
void supabase_client_free(SupabaseClient* client);

// Authentication
SupabaseError supabase_auth_sign_in(
    SupabaseClient* client,
    const char* email,
    const char* password,
    char* result,
    size_t result_len
);

SupabaseError supabase_auth_sign_up(
    SupabaseClient* client,
    const char* email,
    const char* password,
    char* result,
    size_t result_len
);

// Database operations
SupabaseError supabase_database_select(
    SupabaseClient* client,
    const char* table,
    const char* columns,
    char* result,
    size_t result_len
);

SupabaseError supabase_database_insert(
    SupabaseClient* client,
    const char* table,
    const char* json_data,
    char* result,
    size_t result_len
);

// Storage operations
SupabaseError supabase_storage_list_buckets(
    SupabaseClient* client,
    char* result,
    size_t result_len
);

// Edge Functions
SupabaseError supabase_functions_invoke(
    SupabaseClient* client,
    const char* function_name,
    const char* json_payload,
    char* result,
    size_t result_len
);

// Error handling
SupabaseError supabase_get_last_error(char* buffer, size_t buffer_len);

#ifdef __cplusplus
}
#endif

#endif // SUPABASE_H
