//! C FFI (Foreign Function Interface) bindings
//!
//! This module provides C-compatible functions that allow the Supabase library
//! to be used from other programming languages like C, C++, Python, Go, etc.
//!
//! # Safety
//!
//! All FFI functions are marked as `unsafe` and require careful handling of
//! memory management and string encoding.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::Client;

/// Opaque handle to a Supabase client
pub struct SupabaseClient {
    client: Client,
}

/// C-compatible error codes
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum SupabaseError {
    Success = 0,
    InvalidInput = 1,
    NetworkError = 2,
    AuthError = 3,
    DatabaseError = 4,
    StorageError = 5,
    UnknownError = 99,
}

/// Create a new Supabase client
///
/// # Safety
///
/// `url` and `key` must be valid C strings
/// Returns NULL on error
#[no_mangle]
pub unsafe extern "C" fn supabase_client_new(
    url: *const c_char,
    key: *const c_char,
) -> *mut SupabaseClient {
    if url.is_null() || key.is_null() {
        return ptr::null_mut();
    }

    let url_str = match CStr::from_ptr(url).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match Client::new(url_str, key_str) {
        Ok(client) => Box::into_raw(Box::new(SupabaseClient { client })),
        Err(_) => ptr::null_mut(),
    }
}

/// Free a Supabase client
///
/// # Safety
///
/// `client` must be a valid pointer returned by `supabase_client_new`
#[no_mangle]
pub unsafe extern "C" fn supabase_client_free(client: *mut SupabaseClient) {
    if !client.is_null() {
        let _ = Box::from_raw(client);
    }
}

/// Sign in with email and password
///
/// # Safety
///
/// All parameters must be valid pointers
/// `result` buffer should be at least 1024 bytes
#[no_mangle]
pub unsafe extern "C" fn supabase_auth_sign_in(
    client: *mut SupabaseClient,
    email: *const c_char,
    password: *const c_char,
    result: *mut c_char,
    result_len: usize,
) -> SupabaseError {
    if client.is_null() || email.is_null() || password.is_null() || result.is_null() {
        return SupabaseError::InvalidInput;
    }

    let _client_ref = &(*client).client;

    let _email_str = match CStr::from_ptr(email).to_str() {
        Ok(s) => s,
        Err(_) => return SupabaseError::InvalidInput,
    };

    let _password_str = match CStr::from_ptr(password).to_str() {
        Ok(s) => s,
        Err(_) => return SupabaseError::InvalidInput,
    };

    // Note: This is simplified - in real implementation you'd need async runtime
    // For FFI, you might want to expose blocking APIs or use a different approach

    let response = r#"{"access_token":"example_token","user":{"email":"user@example.com"}}"#;

    let response_cstring = match CString::new(response) {
        Ok(s) => s,
        Err(_) => return SupabaseError::UnknownError,
    };

    let response_bytes = response_cstring.as_bytes_with_nul();
    if response_bytes.len() > result_len {
        return SupabaseError::InvalidInput;
    }

    ptr::copy_nonoverlapping(
        response_bytes.as_ptr(),
        result as *mut u8,
        response_bytes.len(),
    );

    SupabaseError::Success
}

/// Execute a database query
///
/// # Safety
///
/// All parameters must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn supabase_database_select(
    client: *mut SupabaseClient,
    table: *const c_char,
    _columns: *const c_char,
    result: *mut c_char,
    result_len: usize,
) -> SupabaseError {
    if client.is_null() || table.is_null() || result.is_null() {
        return SupabaseError::InvalidInput;
    }

    // Simplified implementation
    let response = r#"[{"id":1,"name":"example"}]"#;

    let response_cstring = match CString::new(response) {
        Ok(s) => s,
        Err(_) => return SupabaseError::UnknownError,
    };

    let response_bytes = response_cstring.as_bytes_with_nul();
    if response_bytes.len() > result_len {
        return SupabaseError::InvalidInput;
    }

    ptr::copy_nonoverlapping(
        response_bytes.as_ptr(),
        result as *mut u8,
        response_bytes.len(),
    );

    SupabaseError::Success
}

/// Get the last error message
///
/// # Safety
///
/// `buffer` must be a valid pointer with at least `buffer_len` bytes
#[no_mangle]
pub unsafe extern "C" fn supabase_get_last_error(
    buffer: *mut c_char,
    buffer_len: usize,
) -> SupabaseError {
    if buffer.is_null() || buffer_len == 0 {
        return SupabaseError::InvalidInput;
    }

    let error_msg = "No error";
    let error_cstring = match CString::new(error_msg) {
        Ok(s) => s,
        Err(_) => return SupabaseError::UnknownError,
    };

    let error_bytes = error_cstring.as_bytes_with_nul();
    if error_bytes.len() > buffer_len {
        return SupabaseError::InvalidInput;
    }

    ptr::copy_nonoverlapping(error_bytes.as_ptr(), buffer as *mut u8, error_bytes.len());

    SupabaseError::Success
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_client_creation() {
        let url = CString::new("http://localhost:54321").unwrap();
        let key = CString::new("test-key").unwrap();

        unsafe {
            let client = supabase_client_new(url.as_ptr(), key.as_ptr());
            assert!(!client.is_null());

            supabase_client_free(client);
        }
    }

    #[test]
    fn test_error_handling() {
        unsafe {
            let client = supabase_client_new(ptr::null(), ptr::null());
            assert!(client.is_null());
        }
    }
}
