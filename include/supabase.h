#ifndef SUPABASE_H
#define SUPABASE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

/* Opaque handle to Supabase client */
typedef struct SupabaseClient SupabaseClient;

/* Error codes */
typedef enum {
    SUPABASE_SUCCESS = 0,
    SUPABASE_INVALID_INPUT = 1,
    SUPABASE_NETWORK_ERROR = 2,
    SUPABASE_AUTH_ERROR = 3,
    SUPABASE_DATABASE_ERROR = 4,
    SUPABASE_STORAGE_ERROR = 5,
    SUPABASE_UNKNOWN_ERROR = 99
} SupabaseError;

/* Client management */
SupabaseClient* supabase_client_new(const char* url, const char* key);
void supabase_client_free(SupabaseClient* client);

/* Authentication */
SupabaseError supabase_auth_sign_in(
    SupabaseClient* client,
    const char* email,
    const char* password,
    char* result,
    size_t result_len
);

/* Database operations */
SupabaseError supabase_database_select(
    SupabaseClient* client,
    const char* table,
    const char* columns,
    char* result,
    size_t result_len
);

/* Error handling */
SupabaseError supabase_get_last_error(char* buffer, size_t buffer_len);

/* Version information */
const char* supabase_version(void);

#ifdef __cplusplus
}
#endif

#endif /* SUPABASE_H */
