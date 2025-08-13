//! Authentication module for Supabase

use crate::{
    error::{Error, Result},
    types::{SupabaseConfig, Timestamp},
};
use chrono::Utc;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Authentication client for handling user sessions and JWT tokens
#[derive(Debug, Clone)]
pub struct Auth {
    http_client: Arc<HttpClient>,
    config: Arc<SupabaseConfig>,
    session: Arc<RwLock<Option<Session>>>,
}

/// User information from Supabase Auth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub email_confirmed_at: Option<Timestamp>,
    pub phone_confirmed_at: Option<Timestamp>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub last_sign_in_at: Option<Timestamp>,
    pub app_metadata: serde_json::Value,
    pub user_metadata: serde_json::Value,
    pub aud: String,
    pub role: Option<String>,
}

/// Authentication session containing user and tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub expires_at: Timestamp,
    pub token_type: String,
    pub user: User,
}

/// Response from authentication operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: Option<User>,
    pub session: Option<Session>,
}

/// Sign up request payload
#[derive(Debug, Serialize)]
struct SignUpRequest {
    email: String,
    password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_to: Option<String>,
}

/// Sign in request payload
#[derive(Debug, Serialize)]
struct SignInRequest {
    email: String,
    password: String,
}

/// Password reset request payload
#[derive(Debug, Serialize)]
struct PasswordResetRequest {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_to: Option<String>,
}

/// Token refresh request payload
#[derive(Debug, Serialize)]
struct RefreshTokenRequest {
    refresh_token: String,
}

/// Update user request payload
#[derive(Debug, Serialize)]
struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

impl Auth {
    /// Create a new Auth instance
    pub fn new(config: Arc<SupabaseConfig>, http_client: Arc<HttpClient>) -> Result<Self> {
        debug!("Initializing Auth module");

        Ok(Self {
            http_client,
            config,
            session: Arc::new(RwLock::new(None)),
        })
    }

    /// Sign up a new user with email and password
    pub async fn sign_up_with_email_and_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse> {
        self.sign_up_with_email_password_and_data(email, password, None, None)
            .await
    }

    /// Sign up a new user with email, password, and optional metadata
    pub async fn sign_up_with_email_password_and_data(
        &self,
        email: &str,
        password: &str,
        data: Option<serde_json::Value>,
        redirect_to: Option<String>,
    ) -> Result<AuthResponse> {
        debug!("Signing up user with email: {}", email);

        let payload = SignUpRequest {
            email: email.to_string(),
            password: password.to_string(),
            data,
            redirect_to,
        };

        let response = self
            .http_client
            .post(format!("{}/auth/v1/signup", self.config.url))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Sign up failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        let auth_response: AuthResponse = response.json().await?;

        if let Some(ref session) = auth_response.session {
            self.set_session(session.clone()).await?;
            info!("User signed up successfully");
        }

        Ok(auth_response)
    }

    /// Sign in with email and password
    pub async fn sign_in_with_email_and_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse> {
        debug!("Signing in user with email: {}", email);

        let payload = SignInRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self
            .http_client
            .post(format!("{}/auth/v1/token?grant_type=password", self.config.url))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Sign in failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        let auth_response: AuthResponse = response.json().await?;

        if let Some(ref session) = auth_response.session {
            self.set_session(session.clone()).await?;
            info!("User signed in successfully");
        }

        Ok(auth_response)
    }

    /// Sign out the current user
    pub async fn sign_out(&self) -> Result<()> {
        debug!("Signing out user");

        let session = self.get_session()?;

        let response = self
            .http_client
            .post(format!("{}/auth/v1/logout", self.config.url))
            .header("Authorization", format!("Bearer {}", session.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Sign out request failed with status: {}", response.status());
        }

        self.clear_session().await?;
        info!("User signed out successfully");

        Ok(())
    }

    /// Reset password via email
    pub async fn reset_password_for_email(&self, email: &str) -> Result<()> {
        self.reset_password_for_email_with_redirect(email, None)
            .await
    }

    /// Reset password via email with optional redirect URL
    pub async fn reset_password_for_email_with_redirect(
        &self,
        email: &str,
        redirect_to: Option<String>,
    ) -> Result<()> {
        debug!("Requesting password reset for email: {}", email);

        let payload = PasswordResetRequest {
            email: email.to_string(),
            redirect_to,
        };

        let response = self
            .http_client
            .post(format!("{}/auth/v1/recover", self.config.url))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Password reset failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        info!("Password reset email sent successfully");
        Ok(())
    }

    /// Update the current user's information
    pub async fn update_user(
        &self,
        email: Option<String>,
        password: Option<String>,
        data: Option<serde_json::Value>,
    ) -> Result<AuthResponse> {
        debug!("Updating user information");

        let session = self.get_session()?;

        let payload = UpdateUserRequest {
            email,
            password,
            data,
        };

        let response = self
            .http_client
            .put(format!("{}/auth/v1/user", self.config.url))
            .header("Authorization", format!("Bearer {}", session.access_token))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("User update failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        let auth_response: AuthResponse = response.json().await?;

        if let Some(ref session) = auth_response.session {
            self.set_session(session.clone()).await?;
        }

        info!("User updated successfully");
        Ok(auth_response)
    }

    /// Refresh the current session token
    pub async fn refresh_session(&self) -> Result<AuthResponse> {
        debug!("Refreshing session token");

        let current_session = self.get_session()?;

        let payload = RefreshTokenRequest {
            refresh_token: current_session.refresh_token.clone(),
        };

        let response = self
            .http_client
            .post(format!("{}/auth/v1/token?grant_type=refresh_token", self.config.url))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Token refresh failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        let auth_response: AuthResponse = response.json().await?;

        if let Some(ref session) = auth_response.session {
            self.set_session(session.clone()).await?;
            info!("Session refreshed successfully");
        }

        Ok(auth_response)
    }

    /// Get the current user information
    pub async fn current_user(&self) -> Result<Option<User>> {
        let session_guard = self
            .session
            .read()
            .map_err(|_| Error::auth("Failed to read session"))?;
        Ok(session_guard.as_ref().map(|s| s.user.clone()))
    }

    /// Get the current session
    pub fn get_session(&self) -> Result<Session> {
        let session_guard = self
            .session
            .read()
            .map_err(|_| Error::auth("Failed to read session"))?;
        session_guard
            .as_ref()
            .cloned()
            .ok_or_else(|| Error::auth("No active session"))
    }

    /// Set a new session
    pub async fn set_session(&self, session: Session) -> Result<()> {
        let mut session_guard = self
            .session
            .write()
            .map_err(|_| Error::auth("Failed to write session"))?;
        *session_guard = Some(session);
        Ok(())
    }

    /// Set session from JWT token
    pub async fn set_session_token(&self, token: &str) -> Result<()> {
        debug!("Setting session from token");

        let user_response = self
            .http_client
            .get(format!("{}/auth/v1/user", self.config.url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !user_response.status().is_success() {
            return Err(Error::auth("Invalid token"));
        }

        let user: User = user_response.json().await?;

        let session = Session {
            access_token: token.to_string(),
            refresh_token: String::new(),
            expires_in: 3600,
            expires_at: Utc::now() + chrono::Duration::seconds(3600),
            token_type: "bearer".to_string(),
            user,
        };

        self.set_session(session).await?;
        Ok(())
    }

    /// Clear the current session
    pub async fn clear_session(&self) -> Result<()> {
        let mut session_guard = self
            .session
            .write()
            .map_err(|_| Error::auth("Failed to write session"))?;
        *session_guard = None;
        Ok(())
    }

    /// Check if the user is authenticated
    pub fn is_authenticated(&self) -> bool {
        let session_guard = self.session.read().unwrap_or_else(|_| {
            warn!("Failed to read session lock");
            self.session.read().unwrap()
        });

        match session_guard.as_ref() {
            Some(session) => {
                let now = Utc::now();
                session.expires_at > now
            }
            None => false,
        }
    }

    /// Check if the current token needs refresh
    pub fn needs_refresh(&self) -> bool {
        let session_guard = match self.session.read() {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        match session_guard.as_ref() {
            Some(session) => {
                let now = Utc::now();
                let threshold =
                    chrono::Duration::seconds(self.config.auth_config.refresh_threshold as i64);
                session.expires_at - now < threshold
            }
            None => false,
        }
    }

    /// Auto-refresh token if needed
    pub async fn auto_refresh(&self) -> Result<()> {
        if !self.config.auth_config.auto_refresh_token || !self.needs_refresh() {
            return Ok(());
        }

        debug!("Auto-refreshing token");
        self.refresh_session().await.map(|_| ())
    }
}
