//! Authentication module for Supabase client
//!
//! This module provides comprehensive authentication functionality including:
//! - Email/password authentication
//! - OAuth providers (Google, GitHub, Discord, Apple, etc.)
//! - Phone/SMS authentication
//! - Magic link authentication
//! - Anonymous authentication
//! - Session management and token refresh
//! - Auth state change events

use crate::{
    error::{Error, Result},
    types::{SupabaseConfig, Timestamp},
};
use chrono::Utc;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Authentication state event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthEvent {
    /// User signed in
    SignedIn,
    /// User signed out
    SignedOut,
    /// Session refreshed
    TokenRefreshed,
    /// User information updated
    UserUpdated,
    /// Password reset initiated
    PasswordReset,
}

/// Authentication state change callback
pub type AuthStateCallback = Box<dyn Fn(AuthEvent, Option<Session>) + Send + Sync + 'static>;

/// Authentication event listener handle
#[derive(Debug, Clone)]
pub struct AuthEventHandle {
    id: Uuid,
    auth: Weak<Auth>,
}

impl AuthEventHandle {
    /// Remove this event listener
    pub fn remove(&self) {
        if let Some(auth) = self.auth.upgrade() {
            auth.remove_auth_listener(self.id);
        }
    }
}

/// Supported OAuth providers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OAuthProvider {
    /// Google OAuth
    #[serde(rename = "google")]
    Google,
    /// GitHub OAuth
    #[serde(rename = "github")]
    GitHub,
    /// Discord OAuth
    #[serde(rename = "discord")]
    Discord,
    /// Apple OAuth
    #[serde(rename = "apple")]
    Apple,
    /// Twitter OAuth
    #[serde(rename = "twitter")]
    Twitter,
    /// Facebook OAuth
    #[serde(rename = "facebook")]
    Facebook,
    /// Microsoft OAuth
    #[serde(rename = "azure")]
    Microsoft,
    /// LinkedIn OAuth
    #[serde(rename = "linkedin_oidc")]
    LinkedIn,
}

impl OAuthProvider {
    /// Get provider name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::GitHub => "github",
            OAuthProvider::Discord => "discord",
            OAuthProvider::Apple => "apple",
            OAuthProvider::Twitter => "twitter",
            OAuthProvider::Facebook => "facebook",
            OAuthProvider::Microsoft => "azure",
            OAuthProvider::LinkedIn => "linkedin_oidc",
        }
    }
}

/// OAuth sign-in options
#[derive(Debug, Clone, Default)]
pub struct OAuthOptions {
    /// Custom redirect URL after sign-in
    pub redirect_to: Option<String>,
    /// Additional scopes to request
    pub scopes: Option<Vec<String>>,
    /// Additional provider-specific options
    pub query_params: Option<std::collections::HashMap<String, String>>,
}

/// OAuth response with authorization URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthResponse {
    /// Authorization URL to redirect user to
    pub url: String,
}

/// Phone authentication request
#[derive(Debug, Serialize)]
struct PhoneSignUpRequest {
    phone: String,
    password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// Phone authentication sign-in request
#[derive(Debug, Serialize)]
struct PhoneSignInRequest {
    phone: String,
    password: String,
}

/// OTP verification request
#[derive(Debug, Serialize)]
struct OTPVerificationRequest {
    phone: String,
    token: String,
    #[serde(rename = "type")]
    verification_type: String,
}

/// Magic link request
#[derive(Debug, Serialize)]
struct MagicLinkRequest {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// Anonymous sign-in request
#[derive(Debug, Serialize)]
struct AnonymousSignInRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// Authentication client for handling user sessions and JWT tokens
pub struct Auth {
    http_client: Arc<HttpClient>,
    config: Arc<SupabaseConfig>,
    session: Arc<RwLock<Option<Session>>>,
    event_listeners: Arc<RwLock<HashMap<Uuid, AuthStateCallback>>>,
}

impl Clone for Auth {
    fn clone(&self) -> Self {
        Self {
            http_client: self.http_client.clone(),
            config: self.config.clone(),
            session: self.session.clone(),
            event_listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl std::fmt::Debug for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Auth")
            .field("http_client", &"HttpClient")
            .field("config", &self.config)
            .field("session", &self.session)
            .field(
                "event_listeners",
                &format!(
                    "HashMap<Uuid, Callback> with {} listeners",
                    self.event_listeners.read().map(|l| l.len()).unwrap_or(0)
                ),
            )
            .finish()
    }
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
            event_listeners: Arc::new(RwLock::new(HashMap::new())),
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
            self.trigger_auth_event(AuthEvent::SignedIn);
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
            .post(format!(
                "{}/auth/v1/token?grant_type=password",
                self.config.url
            ))
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
            self.trigger_auth_event(AuthEvent::SignedIn);
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
        self.trigger_auth_event(AuthEvent::SignedOut);
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
            .post(format!(
                "{}/auth/v1/token?grant_type=refresh_token",
                self.config.url
            ))
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
            self.trigger_auth_event(AuthEvent::TokenRefreshed);
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
            Err(_) => {
                warn!("Failed to read session lock");
                return false;
            }
        };

        match session_guard.as_ref() {
            Some(session) => {
                let now = Utc::now();
                let buffer = chrono::Duration::minutes(5); // Refresh 5 minutes before expiry
                session.expires_at < (now + buffer)
            }
            None => false,
        }
    }

    /// Sign in with OAuth provider
    ///
    /// Returns a URL that the user should be redirected to for authentication.
    /// After successful authentication, the user will be redirected back with the session.
    ///
    /// # Example
    ///
    /// ```rust
    /// use supabase::auth::{OAuthProvider, OAuthOptions};
    ///
    /// # async fn example() -> supabase::Result<()> {
    /// let client = supabase::Client::new("url", "key")?;
    ///
    /// let options = OAuthOptions {
    ///     redirect_to: Some("https://myapp.com/callback".to_string()),
    ///     scopes: Some(vec!["email".to_string(), "profile".to_string()]),
    ///     ..Default::default()
    /// };
    ///
    /// let response = client.auth().sign_in_with_oauth(OAuthProvider::Google, Some(options)).await?;
    /// println!("Redirect to: {}", response.url);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sign_in_with_oauth(
        &self,
        provider: OAuthProvider,
        options: Option<OAuthOptions>,
    ) -> Result<OAuthResponse> {
        debug!("Initiating OAuth sign-in with provider: {:?}", provider);

        let mut url = format!(
            "{}/auth/v1/authorize?provider={}",
            self.config.url,
            provider.as_str()
        );

        if let Some(opts) = options {
            if let Some(redirect_to) = opts.redirect_to {
                url.push_str(&format!(
                    "&redirect_to={}",
                    urlencoding::encode(&redirect_to)
                ));
            }

            if let Some(scopes) = opts.scopes {
                let scope_str = scopes.join(" ");
                url.push_str(&format!("&scope={}", urlencoding::encode(&scope_str)));
            }

            if let Some(query_params) = opts.query_params {
                for (key, value) in query_params {
                    url.push_str(&format!(
                        "&{}={}",
                        urlencoding::encode(&key),
                        urlencoding::encode(&value)
                    ));
                }
            }
        }

        Ok(OAuthResponse { url })
    }

    /// Sign up with phone number
    ///
    /// # Example
    ///
    /// ```rust
    /// # async fn example() -> supabase::Result<()> {
    /// let client = supabase::Client::new("url", "key")?;
    ///
    /// let response = client.auth()
    ///     .sign_up_with_phone("+1234567890", "securepassword", None)
    ///     .await?;
    ///
    /// if let Some(user) = response.user {
    ///     println!("User created: {:?}", user.phone);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sign_up_with_phone(
        &self,
        phone: &str,
        password: &str,
        data: Option<serde_json::Value>,
    ) -> Result<AuthResponse> {
        debug!("Signing up user with phone: {}", phone);

        let payload = PhoneSignUpRequest {
            phone: phone.to_string(),
            password: password.to_string(),
            data,
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
                Err(_) => format!("Phone sign up failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        let auth_response: AuthResponse = response.json().await?;

        if let Some(ref session) = auth_response.session {
            self.set_session(session.clone()).await?;
            self.trigger_auth_event(AuthEvent::SignedIn);
            info!("User signed up with phone successfully");
        }

        Ok(auth_response)
    }

    /// Sign in with phone number
    ///
    /// # Example
    ///
    /// ```rust
    /// # async fn example() -> supabase::Result<()> {
    /// let client = supabase::Client::new("url", "key")?;
    ///
    /// let response = client.auth()
    ///     .sign_in_with_phone("+1234567890", "securepassword")
    ///     .await?;
    ///
    /// if let Some(user) = response.user {
    ///     println!("User signed in: {:?}", user.phone);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sign_in_with_phone(&self, phone: &str, password: &str) -> Result<AuthResponse> {
        debug!("Signing in user with phone: {}", phone);

        let payload = PhoneSignInRequest {
            phone: phone.to_string(),
            password: password.to_string(),
        };

        let response = self
            .http_client
            .post(format!(
                "{}/auth/v1/token?grant_type=password",
                self.config.url
            ))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Phone sign in failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        let auth_response: AuthResponse = response.json().await?;

        if let Some(ref session) = auth_response.session {
            self.set_session(session.clone()).await?;
            self.trigger_auth_event(AuthEvent::SignedIn);
            info!("User signed in with phone successfully");
        }

        Ok(auth_response)
    }

    /// Verify OTP token
    ///
    /// # Example
    ///
    /// ```rust
    /// # async fn example() -> supabase::Result<()> {
    /// let client = supabase::Client::new("url", "key")?;
    ///
    /// let response = client.auth()
    ///     .verify_otp("+1234567890", "123456", "sms")
    ///     .await?;
    ///
    /// if let Some(session) = response.session {
    ///     println!("OTP verified, user signed in");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn verify_otp(
        &self,
        phone: &str,
        token: &str,
        verification_type: &str,
    ) -> Result<AuthResponse> {
        debug!("Verifying OTP for phone: {}", phone);

        let payload = OTPVerificationRequest {
            phone: phone.to_string(),
            token: token.to_string(),
            verification_type: verification_type.to_string(),
        };

        let response = self
            .http_client
            .post(format!("{}/auth/v1/verify", self.config.url))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("OTP verification failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        let auth_response: AuthResponse = response.json().await?;

        if let Some(ref session) = auth_response.session {
            self.set_session(session.clone()).await?;
            self.trigger_auth_event(AuthEvent::SignedIn);
            info!("OTP verified successfully");
        }

        Ok(auth_response)
    }

    /// Send magic link for passwordless authentication
    ///
    /// # Example
    ///
    /// ```rust
    /// # async fn example() -> supabase::Result<()> {
    /// let client = supabase::Client::new("url", "key")?;
    ///
    /// client.auth()
    ///     .sign_in_with_magic_link("user@example.com", Some("https://myapp.com/callback".to_string()), None)
    ///     .await?;
    ///
    /// println!("Magic link sent to email");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sign_in_with_magic_link(
        &self,
        email: &str,
        redirect_to: Option<String>,
        data: Option<serde_json::Value>,
    ) -> Result<()> {
        debug!("Sending magic link to email: {}", email);

        let payload = MagicLinkRequest {
            email: email.to_string(),
            redirect_to,
            data,
        };

        let response = self
            .http_client
            .post(format!("{}/auth/v1/magiclink", self.config.url))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Magic link request failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        info!("Magic link sent successfully");
        Ok(())
    }

    /// Sign in anonymously
    ///
    /// Creates a temporary anonymous user session that can be converted to a permanent account later.
    ///
    /// # Example
    ///
    /// ```rust
    /// # async fn example() -> supabase::Result<()> {
    /// let client = supabase::Client::new("url", "key")?;
    ///
    /// let response = client.auth()
    ///     .sign_in_anonymously(None)
    ///     .await?;
    ///
    /// if let Some(user) = response.user {
    ///     println!("Anonymous user created: {}", user.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sign_in_anonymously(
        &self,
        data: Option<serde_json::Value>,
    ) -> Result<AuthResponse> {
        debug!("Creating anonymous user session");

        let payload = AnonymousSignInRequest { data };

        let response = self
            .http_client
            .post(format!("{}/auth/v1/signup", self.config.url))
            .header("Authorization", format!("Bearer {}", self.config.key))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Anonymous sign in failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        let auth_response: AuthResponse = response.json().await?;

        if let Some(ref session) = auth_response.session {
            self.set_session(session.clone()).await?;
            self.trigger_auth_event(AuthEvent::SignedIn);
            info!("Anonymous user session created successfully");
        }

        Ok(auth_response)
    }

    /// Enhanced password recovery with custom redirect and options
    ///
    /// # Example
    ///
    /// ```rust
    /// # async fn example() -> supabase::Result<()> {
    /// let client = supabase::Client::new("url", "key")?;
    ///
    /// client.auth()
    ///     .reset_password_for_email_enhanced("user@example.com", Some("https://myapp.com/reset".to_string()))
    ///     .await?;
    ///
    /// println!("Password reset email sent");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reset_password_for_email_enhanced(
        &self,
        email: &str,
        redirect_to: Option<String>,
    ) -> Result<()> {
        debug!("Initiating enhanced password recovery for email: {}", email);

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
                Err(_) => format!("Enhanced password recovery failed with status: {}", status),
            };
            return Err(Error::auth(error_msg));
        }

        self.trigger_auth_event(AuthEvent::PasswordReset);
        info!("Enhanced password recovery email sent successfully");
        Ok(())
    }

    /// Subscribe to authentication state changes
    ///
    /// Returns a handle that can be used to remove the listener later.
    ///
    /// # Example
    ///
    /// ```rust
    /// use supabase::auth::AuthEvent;
    ///
    /// # async fn example() -> supabase::Result<()> {
    /// let client = supabase::Client::new("url", "key")?;
    ///
    /// let handle = client.auth().on_auth_state_change(|event, session| {
    ///     match event {
    ///         AuthEvent::SignedIn => {
    ///             if let Some(session) = session {
    ///                 println!("User signed in: {}", session.user.email.unwrap_or_default());
    ///             }
    ///         }
    ///         AuthEvent::SignedOut => println!("User signed out"),
    ///         AuthEvent::TokenRefreshed => println!("Token refreshed"),
    ///         _ => {}
    ///     }
    /// });
    ///
    /// // Later remove the listener
    /// handle.remove();
    /// # Ok(())
    /// # }
    /// ```
    pub fn on_auth_state_change<F>(&self, callback: F) -> AuthEventHandle
    where
        F: Fn(AuthEvent, Option<Session>) + Send + Sync + 'static,
    {
        let id = Uuid::new_v4();
        let callback = Box::new(callback);

        if let Ok(mut listeners) = self.event_listeners.write() {
            listeners.insert(id, callback);
        }

        AuthEventHandle {
            id,
            auth: Arc::downgrade(&Arc::new(self.clone())),
        }
    }

    /// Remove an authentication state listener
    pub fn remove_auth_listener(&self, id: Uuid) {
        if let Ok(mut listeners) = self.event_listeners.write() {
            listeners.remove(&id);
        }
    }

    /// Trigger authentication state change event
    fn trigger_auth_event(&self, event: AuthEvent) {
        let session = match self.session.read() {
            Ok(guard) => guard.clone(),
            Err(_) => {
                warn!("Failed to read session for event trigger");
                return;
            }
        };

        let listeners = match self.event_listeners.read() {
            Ok(guard) => guard,
            Err(_) => {
                warn!("Failed to read event listeners");
                return;
            }
        };

        for callback in listeners.values() {
            callback(event.clone(), session.clone());
        }
    }
}
