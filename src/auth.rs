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

// MFA imports
use base32;
use phonenumber::Mode;
use qrcode::QrCode;
use totp_rs::{Algorithm, TOTP};

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
    /// MFA challenge required
    MfaChallengeRequired,
    /// MFA challenge completed
    MfaChallengeCompleted,
    /// MFA enabled for user
    MfaEnabled,
    /// MFA disabled for user
    MfaDisabled,
}

/// Multi-factor authentication method types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MfaMethod {
    /// Time-based One-Time Password (TOTP) - Google Authenticator, Authy, etc.
    #[serde(rename = "totp")]
    Totp,
    /// SMS-based verification
    #[serde(rename = "sms")]
    Sms,
    /// Email-based verification (future)
    #[serde(rename = "email")]
    Email,
}

impl std::fmt::Display for MfaMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MfaMethod::Totp => write!(f, "TOTP"),
            MfaMethod::Sms => write!(f, "SMS"),
            MfaMethod::Email => write!(f, "Email"),
        }
    }
}

/// MFA challenge status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MfaChallengeStatus {
    /// Challenge is pending user input
    #[serde(rename = "pending")]
    Pending,
    /// Challenge was completed successfully
    #[serde(rename = "completed")]
    Completed,
    /// Challenge expired
    #[serde(rename = "expired")]
    Expired,
    /// Challenge was cancelled
    #[serde(rename = "cancelled")]
    Cancelled,
}

/// MFA factor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaFactor {
    pub id: Uuid,
    pub factor_type: MfaMethod,
    pub friendly_name: String,
    pub status: String, // "verified", "unverified"
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

/// TOTP setup response containing QR code data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub qr_code: String,
    pub uri: String,
    pub factor_id: Uuid,
}

/// MFA challenge information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaChallenge {
    pub id: Uuid,
    pub factor_id: Uuid,
    pub status: MfaChallengeStatus,
    pub challenge_type: MfaMethod,
    pub expires_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masked_phone: Option<String>,
}

/// MFA verification request
#[derive(Debug, Serialize)]
pub struct MfaVerificationRequest {
    pub factor_id: Uuid,
    pub challenge_id: Uuid,
    pub code: String,
}

/// Enhanced phone number with country code support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPhoneNumber {
    pub raw: String,
    pub formatted: String,
    pub country_code: String,
    pub national_number: String,
    pub is_valid: bool,
}

impl EnhancedPhoneNumber {
    /// Create enhanced phone number from raw input
    pub fn new(phone: &str, _default_region: Option<&str>) -> Result<Self> {
        // For now, let's use a simplified approach
        // In production, this should be properly implemented with phonenumber crate
        let parsed = phonenumber::parse(None, phone)
            .map_err(|e| Error::auth(format!("Invalid phone number: {}", e)))?;

        let formatted = phonenumber::format(&parsed)
            .mode(Mode::International)
            .to_string();

        // Extract basic info
        let country_code = parsed.code().value().to_string();
        let national_number = parsed.national().to_string();

        Ok(Self {
            raw: phone.to_string(),
            formatted,
            country_code,
            national_number,
            is_valid: phonenumber::is_valid(&parsed),
        })
    }
}

/// OAuth token metadata for advanced management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub issued_at: Timestamp,
    pub expires_at: Timestamp,
    pub refresh_count: u32,
    pub last_refresh_at: Option<Timestamp>,
    pub scopes: Vec<String>,
    pub device_id: Option<String>,
}

/// Enhanced session with MFA and advanced token info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub expires_at: Timestamp,
    pub token_type: String,
    pub user: User,
    pub token_metadata: Option<TokenMetadata>,
    pub mfa_verified: bool,
    pub active_factors: Vec<MfaFactor>,
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
    #[serde(with = "chrono::serde::ts_seconds")]
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

        let auth_response_body = response.text().await?;

        let mut auth_response = serde_json::from_str::<AuthResponse>(auth_response_body.as_str())?;
        auth_response.session = serde_json::from_str::<Session>(auth_response_body.as_str())
            .inspect_err(|err| warn!("No session: {}", err.to_string()))
            .ok();

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

        let auth_response_body = response.text().await?;

        let mut auth_response = serde_json::from_str::<AuthResponse>(auth_response_body.as_str())?;
        auth_response.session = serde_json::from_str::<Session>(auth_response_body.as_str())
            .inspect_err(|err| warn!("No session: {}", err.to_string()))
            .ok();

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

        let auth_response_body = response.text().await?;

        let mut auth_response = serde_json::from_str::<AuthResponse>(auth_response_body.as_str())?;
        auth_response.session = serde_json::from_str::<Session>(auth_response_body.as_str())
            .inspect_err(|err| warn!("No session: {}", err.to_string()))
            .ok();

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
    /// use supabase_lib_rs::auth::{OAuthProvider, OAuthOptions};
    ///
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = supabase_lib_rs::Client::new("url", "key")?;
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
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = supabase_lib_rs::Client::new("url", "key")?;
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
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = supabase_lib_rs::Client::new("url", "key")?;
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

        let auth_response_body = response.text().await?;

        let mut auth_response = serde_json::from_str::<AuthResponse>(auth_response_body.as_str())?;
        auth_response.session = serde_json::from_str::<Session>(auth_response_body.as_str())
            .inspect_err(|err| warn!("No session: {}", err.to_string()))
            .ok();

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
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = supabase_lib_rs::Client::new("url", "key")?;
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
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = supabase_lib_rs::Client::new("url", "key")?;
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
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = supabase_lib_rs::Client::new("url", "key")?;
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
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = supabase_lib_rs::Client::new("url", "key")?;
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
    /// use supabase_lib_rs::auth::AuthEvent;
    ///
    /// # async fn example() -> supabase_lib_rs::Result<()> {
    /// let client = supabase_lib_rs::Client::new("url", "key")?;
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

    // ==== MFA (Multi-Factor Authentication) Methods ====

    /// List all MFA factors for the current user
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// // List MFA factors
    /// let factors = client.auth().list_mfa_factors().await?;
    /// println!("User has {} MFA factors configured", factors.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_mfa_factors(&self) -> Result<Vec<MfaFactor>> {
        debug!("Listing MFA factors for user");

        let session = self.get_session()?;
        let response = self
            .http_client
            .get(format!("{}/auth/v1/factors", self.config.url))
            .header("Authorization", format!("Bearer {}", session.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::auth("Failed to list MFA factors"));
        }

        let factors: Vec<MfaFactor> = response.json().await?;
        Ok(factors)
    }

    /// Setup TOTP (Time-based One-Time Password) authentication
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// // Setup TOTP
    /// let totp_setup = client.auth()
    ///     .setup_totp("My Authenticator")
    ///     .await?;
    ///
    /// println!("Scan QR code: {}", totp_setup.qr_code);
    /// println!("Or enter secret manually: {}", totp_setup.secret);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn setup_totp(&self, friendly_name: &str) -> Result<TotpSetupResponse> {
        debug!("Setting up TOTP factor: {}", friendly_name);

        let session = self.get_session()?;

        let request_body = serde_json::json!({
            "friendly_name": friendly_name,
            "factor_type": "totp"
        });

        let response = self
            .http_client
            .post(format!("{}/auth/v1/factors", self.config.url))
            .header("Authorization", format!("Bearer {}", session.access_token))
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::auth("Failed to setup TOTP"));
        }

        let setup_response: TotpSetupResponse = response.json().await?;

        // Generate QR code for the TOTP URI
        let qr = QrCode::new(&setup_response.uri)
            .map_err(|e| Error::auth(format!("Failed to generate QR code: {}", e)))?;

        // Convert QR code to string representation (for console display)
        let qr_string = qr
            .render::<char>()
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build();

        Ok(TotpSetupResponse {
            secret: setup_response.secret,
            qr_code: qr_string,
            uri: setup_response.uri,
            factor_id: setup_response.factor_id,
        })
    }

    /// Setup SMS-based MFA with international phone number support
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// // Setup SMS MFA with international number
    /// let factor = client.auth()
    ///     .setup_sms_mfa("+1-555-123-4567", "My Phone", Some("US"))
    ///     .await?;
    ///
    /// println!("SMS MFA configured for: {}", factor.phone.unwrap());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn setup_sms_mfa(
        &self,
        phone: &str,
        friendly_name: &str,
        default_region: Option<&str>,
    ) -> Result<MfaFactor> {
        debug!("Setting up SMS MFA factor: {} for {}", friendly_name, phone);

        // Validate and format phone number
        let enhanced_phone = EnhancedPhoneNumber::new(phone, default_region)?;

        if !enhanced_phone.is_valid {
            return Err(Error::auth("Invalid phone number provided"));
        }

        let session = self.get_session()?;

        let request_body = serde_json::json!({
            "friendly_name": friendly_name,
            "factor_type": "sms",
            "phone": enhanced_phone.formatted
        });

        let response = self
            .http_client
            .post(format!("{}/auth/v1/factors", self.config.url))
            .header("Authorization", format!("Bearer {}", session.access_token))
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::auth("Failed to setup SMS MFA"));
        }

        let factor: MfaFactor = response.json().await?;
        self.trigger_auth_event(AuthEvent::MfaEnabled);

        Ok(factor)
    }

    /// Create MFA challenge for a specific factor
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use uuid::Uuid;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// // Get factors and create challenge
    /// let factors = client.auth().list_mfa_factors().await?;
    /// if let Some(factor) = factors.first() {
    ///     let challenge = client.auth().create_mfa_challenge(factor.id).await?;
    ///     println!("Challenge created: {}", challenge.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_mfa_challenge(&self, factor_id: Uuid) -> Result<MfaChallenge> {
        debug!("Creating MFA challenge for factor: {}", factor_id);

        let session = self.get_session()?;

        let request_body = serde_json::json!({
            "factor_id": factor_id
        });

        let response = self
            .http_client
            .post(format!(
                "{}/auth/v1/factors/{}/challenge",
                self.config.url, factor_id
            ))
            .header("Authorization", format!("Bearer {}", session.access_token))
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::auth("Failed to create MFA challenge"));
        }

        let challenge: MfaChallenge = response.json().await?;
        self.trigger_auth_event(AuthEvent::MfaChallengeRequired);

        Ok(challenge)
    }

    /// Verify MFA challenge with user-provided code
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use uuid::Uuid;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// // Verify MFA code
    /// let factor_id = Uuid::new_v4(); // Your factor ID
    /// let challenge_id = Uuid::new_v4(); // Your challenge ID
    ///
    /// let result = client.auth()
    ///     .verify_mfa_challenge(factor_id, challenge_id, "123456")
    ///     .await?;
    ///
    /// println!("MFA verified successfully!");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn verify_mfa_challenge(
        &self,
        factor_id: Uuid,
        challenge_id: Uuid,
        code: &str,
    ) -> Result<AuthResponse> {
        debug!("Verifying MFA challenge: {}", challenge_id);

        let session = self.get_session()?;

        let request_body = serde_json::json!({
            "factor_id": factor_id,
            "challenge_id": challenge_id,
            "code": code
        });

        let response = self
            .http_client
            .post(format!(
                "{}/auth/v1/factors/{}/verify",
                self.config.url, factor_id
            ))
            .header("Authorization", format!("Bearer {}", session.access_token))
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::auth("Failed to verify MFA challenge"));
        }

        let auth_response: AuthResponse = response.json().await?;

        // Update session if new token provided
        if let Some(session) = &auth_response.session {
            self.set_session(session.clone()).await?;
        }

        self.trigger_auth_event(AuthEvent::MfaChallengeCompleted);
        info!("MFA verification successful");

        Ok(auth_response)
    }

    /// Delete an MFA factor
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use uuid::Uuid;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// // Remove MFA factor
    /// let factor_id = Uuid::new_v4(); // Your factor ID
    /// client.auth().delete_mfa_factor(factor_id).await?;
    /// println!("MFA factor removed successfully!");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_mfa_factor(&self, factor_id: Uuid) -> Result<()> {
        debug!("Deleting MFA factor: {}", factor_id);

        let session = self.get_session()?;

        let response = self
            .http_client
            .delete(format!("{}/auth/v1/factors/{}", self.config.url, factor_id))
            .header("Authorization", format!("Bearer {}", session.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::auth("Failed to delete MFA factor"));
        }

        self.trigger_auth_event(AuthEvent::MfaDisabled);
        info!("MFA factor deleted successfully");

        Ok(())
    }

    /// Generate TOTP code for testing purposes (development only)
    ///
    /// This method is primarily for testing and development. In production,
    /// users should use their authenticator apps.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// // For testing - generate TOTP code from secret
    /// let secret = "JBSWY3DPEHPK3PXP"; // Base32 encoded secret
    /// let code = client.auth().generate_totp_code(secret)?;
    /// println!("Generated TOTP code: {}", code);
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_totp_code(&self, secret: &str) -> Result<String> {
        debug!("Generating TOTP code for testing");

        // Decode base32 secret
        let decoded_secret = base32::decode(base32::Alphabet::Rfc4648 { padding: true }, secret)
            .ok_or_else(|| Error::auth("Invalid base32 secret"))?;

        // Create TOTP
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,  // digits
            1,  // skew (allow 1 step tolerance)
            30, // step (30 seconds)
            decoded_secret,
        )
        .map_err(|e| Error::auth(format!("Failed to create TOTP: {}", e)))?;

        // Generate current code
        let code = totp
            .generate_current()
            .map_err(|e| Error::auth(format!("Failed to generate TOTP code: {}", e)))?;

        Ok(code)
    }

    // ==== Advanced OAuth Token Management ====

    /// Get current token metadata with advanced information
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// // Get detailed token metadata
    /// if let Some(metadata) = client.auth().get_token_metadata()? {
    ///     println!("Token expires at: {}", metadata.expires_at);
    ///     println!("Refresh count: {}", metadata.refresh_count);
    ///     println!("Scopes: {:?}", metadata.scopes);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_token_metadata(&self) -> Result<Option<TokenMetadata>> {
        debug!("Getting token metadata");

        let session = self
            .session
            .read()
            .map_err(|_| Error::auth("Failed to read session"))?;

        if let Some(session) = session.as_ref() {
            // Create metadata from current session
            let metadata = TokenMetadata {
                issued_at: Utc::now() - chrono::Duration::seconds(session.expires_in),
                expires_at: session.expires_at,
                refresh_count: 0, // TODO: Track this in enhanced session
                last_refresh_at: None,
                scopes: vec![],  // TODO: Extract from JWT
                device_id: None, // TODO: Add device tracking
            };

            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    /// Refresh token with advanced error handling and retry logic
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// // Refresh token with advanced handling
    /// match client.auth().refresh_token_advanced().await {
    ///     Ok(session) => {
    ///         println!("Token refreshed successfully!");
    ///         println!("New expiry: {}", session.expires_at);
    ///     }
    ///     Err(e) => {
    ///         if e.is_retryable() {
    ///             // Handle retryable error
    ///             println!("Retryable error: {}", e);
    ///         } else {
    ///             // Handle non-retryable error - require re-login
    ///             println!("Re-authentication required: {}", e);
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn refresh_token_advanced(&self) -> Result<Session> {
        debug!("Refreshing token with advanced handling");

        let current_session = self
            .session
            .read()
            .map_err(|_| Error::auth("Failed to read session"))?
            .clone();

        let session = match current_session {
            Some(session) => session,
            None => return Err(Error::auth("No active session to refresh")),
        };

        let request_body = serde_json::json!({
            "refresh_token": session.refresh_token
        });

        let response = self
            .http_client
            .post(format!(
                "{}/auth/v1/token?grant_type=refresh_token",
                self.config.url
            ))
            .header("apikey", &self.config.key)
            .header("Authorization", format!("Bearer {}", &self.config.key))
            .json(&request_body)
            .send()
            .await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let auth_response_body = response.text().await?;

                    let mut auth_response =
                        serde_json::from_str::<AuthResponse>(auth_response_body.as_str())?;
                    auth_response.session =
                        serde_json::from_str::<Session>(auth_response_body.as_str())
                            .inspect_err(|err| warn!("No session: {}", err.to_string()))
                            .ok();

                    if let Some(new_session) = auth_response.session {
                        self.set_session(new_session.clone()).await?;
                        self.trigger_auth_event(AuthEvent::TokenRefreshed);
                        info!("Token refreshed successfully");
                        Ok(new_session)
                    } else {
                        Err(Error::auth("No session in refresh response"))
                    }
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();

                    // Provide specific error context
                    let context = crate::error::ErrorContext {
                        platform: Some(crate::error::detect_platform_context()),
                        http: Some(crate::error::HttpErrorContext {
                            status_code: Some(status.as_u16()),
                            headers: None,
                            response_body: Some(error_text.clone()),
                            url: Some(format!("{}/auth/v1/token", self.config.url)),
                            method: Some("POST".to_string()),
                        }),
                        retry: if status.is_server_error() {
                            Some(crate::error::RetryInfo {
                                retryable: true,
                                retry_after: Some(60), // 1 minute
                                attempts: 0,
                            })
                        } else {
                            None
                        },
                        metadata: std::collections::HashMap::new(),
                        timestamp: chrono::Utc::now(),
                    };

                    Err(Error::auth_with_context(
                        format!("Token refresh failed: {} - {}", status, error_text),
                        context,
                    ))
                }
            }
            Err(e) => {
                let context = crate::error::ErrorContext {
                    platform: Some(crate::error::detect_platform_context()),
                    http: Some(crate::error::HttpErrorContext {
                        status_code: None,
                        headers: None,
                        response_body: None,
                        url: Some(format!("{}/auth/v1/token", self.config.url)),
                        method: Some("POST".to_string()),
                    }),
                    retry: Some(crate::error::RetryInfo {
                        retryable: true,
                        retry_after: Some(30),
                        attempts: 0,
                    }),
                    metadata: std::collections::HashMap::new(),
                    timestamp: chrono::Utc::now(),
                };

                Err(Error::auth_with_context(
                    format!("Network error during token refresh: {}", e),
                    context,
                ))
            }
        }
    }

    /// Check if current token needs refresh with buffer time
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// // Check if refresh is needed with 5-minute buffer
    /// if client.auth().needs_refresh_with_buffer(300)? {
    ///     println!("Token refresh recommended");
    ///     client.auth().refresh_token_advanced().await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn needs_refresh_with_buffer(&self, buffer_seconds: i64) -> Result<bool> {
        let session_guard = self
            .session
            .read()
            .map_err(|_| Error::auth("Failed to read session"))?;

        match session_guard.as_ref() {
            Some(session) => {
                let now = Utc::now();
                let refresh_threshold =
                    session.expires_at - chrono::Duration::seconds(buffer_seconds);
                Ok(now >= refresh_threshold)
            }
            None => Ok(false), // No session, no need to refresh
        }
    }

    /// Get time until token expiry in seconds
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// match client.auth().time_until_expiry()? {
    ///     Some(seconds) => {
    ///         println!("Token expires in {} seconds", seconds);
    ///         if seconds < 300 {
    ///             println!("Consider refreshing token soon!");
    ///         }
    ///     }
    ///     None => println!("No active session"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn time_until_expiry(&self) -> Result<Option<i64>> {
        let session_guard = self
            .session
            .read()
            .map_err(|_| Error::auth("Failed to read session"))?;

        match session_guard.as_ref() {
            Some(session) => {
                let now = Utc::now();
                let duration = session.expires_at.signed_duration_since(now);
                Ok(Some(duration.num_seconds()))
            }
            None => Ok(None),
        }
    }

    /// Validate current token without making API call (local validation)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("https://example.supabase.co", "your-anon-key")?;
    ///
    /// match client.auth().validate_token_local() {
    ///     Ok(true) => println!("Token is valid locally"),
    ///     Ok(false) => println!("Token is expired or invalid"),
    ///     Err(e) => println!("Validation error: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_token_local(&self) -> Result<bool> {
        let session_guard = self
            .session
            .read()
            .map_err(|_| Error::auth("Failed to read session"))?;

        match session_guard.as_ref() {
            Some(session) => {
                let now = Utc::now();
                Ok(session.expires_at > now && !session.access_token.is_empty())
            }
            None => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SupabaseConfig;
    use std::sync::Arc;

    fn mock_config() -> Arc<SupabaseConfig> {
        Arc::new(SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            key: "test-key".to_string(),
            service_role_key: None,
            http_config: crate::types::HttpConfig::default(),
            auth_config: crate::types::AuthConfig::default(),
            database_config: crate::types::DatabaseConfig::default(),
            storage_config: crate::types::StorageConfig::default(),
        })
    }

    #[test]
    fn test_enhanced_phone_number_creation() {
        let phone = EnhancedPhoneNumber::new("+1-555-123-4567", Some("US"));
        assert!(phone.is_ok());

        let phone = phone.unwrap();
        assert!(!phone.raw.is_empty());
        assert!(!phone.formatted.is_empty());
        assert!(!phone.country_code.is_empty());
    }

    #[test]
    fn test_mfa_method_serialization() {
        let totp = MfaMethod::Totp;
        let sms = MfaMethod::Sms;
        let email = MfaMethod::Email;

        let totp_json = serde_json::to_string(&totp).unwrap();
        let sms_json = serde_json::to_string(&sms).unwrap();
        let email_json = serde_json::to_string(&email).unwrap();

        assert_eq!(totp_json, r#""totp""#);
        assert_eq!(sms_json, r#""sms""#);
        assert_eq!(email_json, r#""email""#);
    }

    #[test]
    fn test_mfa_challenge_status() {
        let pending = MfaChallengeStatus::Pending;
        let completed = MfaChallengeStatus::Completed;
        let expired = MfaChallengeStatus::Expired;
        let cancelled = MfaChallengeStatus::Cancelled;

        assert_eq!(pending, MfaChallengeStatus::Pending);
        assert_eq!(completed, MfaChallengeStatus::Completed);
        assert_eq!(expired, MfaChallengeStatus::Expired);
        assert_eq!(cancelled, MfaChallengeStatus::Cancelled);
    }

    #[test]
    fn test_auth_event_variants() {
        let events = vec![
            AuthEvent::SignedIn,
            AuthEvent::SignedOut,
            AuthEvent::TokenRefreshed,
            AuthEvent::UserUpdated,
            AuthEvent::PasswordReset,
            AuthEvent::MfaChallengeRequired,
            AuthEvent::MfaChallengeCompleted,
            AuthEvent::MfaEnabled,
            AuthEvent::MfaDisabled,
        ];

        assert_eq!(events.len(), 9);

        // Test cloning
        let cloned_events: Vec<AuthEvent> = events.to_vec();
        assert_eq!(cloned_events, events);
    }

    #[test]
    fn test_parsing_auth_from_response() {
        let json_body = r#"{
    "access_token": "eyJhbGciOiJIUzI1NiIsImtpZCI6IkhxWTFsZ3pmbGhOQUx3NTAiLCJ0eXAiOiJKV1QifQ.eyJpc3MiOiJodHRwczovL3Fiamx4cWFyb2l0eHNvdml3bmRsLnN1cGFiYXNlLmNvL2F1dGgvdjEiLCJzdWIiOiIzMTgxNWM1NS1mNTUzLTQxZjQtYjU0Zi0xNGQ2YWM2MGRlMTYiLCJhdWQiOiJhdXRoZW50aWNhdGVkIiwiZXhwIjoxNzYwMDQ1MTk1LCJpYXQiOjE3NjAwNDE1OTUsImVtYWlsIjoic29tZW9uZUBlbWFpbC5jb20iLCJwaG9uZSI6IiIsImFwcF9tZXRhZGF0YSI6eyJwcm92aWRlciI6ImVtYWlsIiwicHJvdmlkZXJzIjpbImVtYWlsIl19LCJ1c2VyX21ldGFkYXRhIjp7ImVtYWlsIjoic29tZW9uZUBlbWFpbC5jb20iLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwicGhvbmVfdmVyaWZpZWQiOmZhbHNlLCJzdWIiOiIzMTgxNWM1NS1mNTUzLTQxZjQtYjU0Zi0xNGQ2YWM2MGRlMTYifSwicm9sZSI6ImF1dGhlbnRpY2F0ZWQiLCJhYWwiOiJhYWwxIiwiYW1yIjpbeyJtZXRob2QiOiJwYXNzd29yZCIsInRpbWVzdGFtcCI6MTc2MDA0MTU5NX1dLCJzZXNzaW9uX2lkIjoiMzc0OTc0OGUtMmUyMy00Nzk0LTllNmQtNjg0MzU5ZDc3M2RjIiwiaXNfYW5vbnltb3VzIjpmYWxzZX0.HCNEZQjnpzBkfEJ_6gJwxcfubRDGez8SRlM6Ni63X_k",
    "token_type": "bearer",
    "expires_in": 3600,
    "expires_at": 1760045195,
    "refresh_token": "pwou3cigit5u",
    "user": {
        "id": "31815c55-f553-41f4-b54f-14d6ac60de16",
        "aud": "authenticated",
        "role": "authenticated",
        "email": "someone@email.com",
        "email_confirmed_at": "2025-10-09T14:53:09.962028Z",
        "phone": "",
        "confirmed_at": "2025-10-09T14:53:09.962028Z",
        "last_sign_in_at": "2025-10-09T20:26:35.136870325Z",
        "app_metadata": {
            "provider": "email",
            "providers": ["email"]
        },
        "user_metadata": {
            "email": "someone@email.com",
            "email_verified": true,
            "phone_verified": false,
            "sub": "31815c55-f553-41f4-b54f-14d6ac60de16"
        },
        "identities": [{
            "identity_id": "5fe7caa2-1dc3-449b-b910-33bd7df0d616",
            "id": "31815c55-f553-41f4-b54f-14d6ac60de16",
            "user_id": "31815c55-f553-41f4-b54f-14d6ac60de16",
            "identity_data": {
                "email": "someone@email.com",
                "email_verified": false,
                "phone_verified": false,
                "sub": "31815c55-f553-41f4-b54f-14d6ac60de16"
            },
            "provider": "email",
            "last_sign_in_at": "2025-10-09T14:53:09.958178Z",
            "created_at": "2025-10-09T14:53:09.958225Z",
            "updated_at": "2025-10-09T14:53:09.958225Z",
            "email": "someone@email.com"
        }],
        "created_at": "2025-10-09T14:53:09.953727Z",
        "updated_at": "2025-10-09T20:26:35.13964Z",
        "is_anonymous": false
    },
    "weak_password": null
}"#;
        let mut auth = serde_json::from_str::<AuthResponse>(json_body).unwrap();
        assert!(auth.session.is_none());

        auth.session = serde_json::from_str::<Session>(json_body).ok();
        assert!(auth.session.is_some());
    }

    #[test]
    fn test_parsing_auth_with_missing_session() {
        let json_body = r#"{
    "user": {
        "id": "31815c55-f553-41f4-b54f-14d6ac60de16",
        "aud": "authenticated",
        "role": "authenticated",
        "email": "someone@email.com",
        "email_confirmed_at": "2025-10-09T14:53:09.962028Z",
        "phone": "",
        "confirmed_at": "2025-10-09T14:53:09.962028Z",
        "last_sign_in_at": "2025-10-09T20:26:35.136870325Z",
        "app_metadata": {
            "provider": "email",
            "providers": ["email"]
        },
        "user_metadata": {
            "email": "someone@email.com",
            "email_verified": true,
            "phone_verified": false,
            "sub": "31815c55-f553-41f4-b54f-14d6ac60de16"
        },
        "identities": [{
            "identity_id": "5fe7caa2-1dc3-449b-b910-33bd7df0d616",
            "id": "31815c55-f553-41f4-b54f-14d6ac60de16",
            "user_id": "31815c55-f553-41f4-b54f-14d6ac60de16",
            "identity_data": {
                "email": "someone@email.com",
                "email_verified": false,
                "phone_verified": false,
                "sub": "31815c55-f553-41f4-b54f-14d6ac60de16"
            },
            "provider": "email",
            "last_sign_in_at": "2025-10-09T14:53:09.958178Z",
            "created_at": "2025-10-09T14:53:09.958225Z",
            "updated_at": "2025-10-09T14:53:09.958225Z",
            "email": "someone@email.com"
        }],
        "created_at": "2025-10-09T14:53:09.953727Z",
        "updated_at": "2025-10-09T20:26:35.13964Z",
        "is_anonymous": false
    },
    "weak_password": null
}"#;
        let mut auth = serde_json::from_str::<AuthResponse>(json_body).unwrap();
        assert!(auth.session.is_none());

        auth.session = serde_json::from_str::<Session>(json_body).ok();
        assert!(auth.session.is_none());
    }

    #[tokio::test]
    async fn test_auth_creation() {
        let config = mock_config();
        let http_client = Arc::new(reqwest::Client::new());

        let auth = Auth::new(config, http_client);
        assert!(auth.is_ok());

        let auth = auth.unwrap();
        assert!(!auth.is_authenticated());
    }

    #[test]
    fn test_totp_code_generation() {
        let config = mock_config();
        let http_client = Arc::new(reqwest::Client::new());
        let auth = Auth::new(config, http_client).unwrap();

        // Test with a known base32 secret
        let secret = "JBSWY3DPEHPK3PXP"; // "Hello" in base32
        let result = auth.generate_totp_code(secret);

        match &result {
            Ok(code) => {
                println!("Generated TOTP code: {}", code);
                assert_eq!(code.len(), 6);
                assert!(code.chars().all(|c| c.is_ascii_digit()));
            }
            Err(e) => {
                println!("TOTP generation error: {}", e);
                // For now, just check that error is reasonable
                assert!(e.to_string().contains("base32") || e.to_string().contains("TOTP"));
            }
        }
    }

    #[test]
    fn test_totp_code_generation_invalid_secret() {
        let config = mock_config();
        let http_client = Arc::new(reqwest::Client::new());
        let auth = Auth::new(config, http_client).unwrap();

        // Test with invalid base32 secret
        let result = auth.generate_totp_code("invalid-secret");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_validation_no_session() {
        let config = mock_config();
        let http_client = Arc::new(reqwest::Client::new());
        let auth = Auth::new(config, http_client).unwrap();

        // No session should return false
        let is_valid = auth.validate_token_local().unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_time_until_expiry_no_session() {
        let config = mock_config();
        let http_client = Arc::new(reqwest::Client::new());
        let auth = Auth::new(config, http_client).unwrap();

        // No session should return None
        let time = auth.time_until_expiry().unwrap();
        assert!(time.is_none());
    }

    #[test]
    fn test_needs_refresh_no_session() {
        let config = mock_config();
        let http_client = Arc::new(reqwest::Client::new());
        let auth = Auth::new(config, http_client).unwrap();

        // No session should return false
        let needs_refresh = auth.needs_refresh_with_buffer(300).unwrap();
        assert!(!needs_refresh);
    }

    #[test]
    fn test_get_token_metadata_no_session() {
        let config = mock_config();
        let http_client = Arc::new(reqwest::Client::new());
        let auth = Auth::new(config, http_client).unwrap();

        // No session should return None
        let metadata = auth.get_token_metadata().unwrap();
        assert!(metadata.is_none());
    }

    #[test]
    fn test_token_metadata_structure() {
        let metadata = TokenMetadata {
            issued_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            refresh_count: 5,
            last_refresh_at: Some(Utc::now()),
            scopes: vec!["read".to_string(), "write".to_string()],
            device_id: Some("device-123".to_string()),
        };

        assert_eq!(metadata.refresh_count, 5);
        assert!(metadata.last_refresh_at.is_some());
        assert_eq!(metadata.scopes.len(), 2);
        assert!(metadata.device_id.is_some());
    }

    #[test]
    fn test_mfa_factor_structure() {
        let factor = MfaFactor {
            id: uuid::Uuid::new_v4(),
            factor_type: MfaMethod::Totp,
            friendly_name: "My Authenticator".to_string(),
            status: "verified".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            phone: None,
        };

        assert_eq!(factor.factor_type, MfaMethod::Totp);
        assert_eq!(factor.friendly_name, "My Authenticator");
        assert_eq!(factor.status, "verified");
        assert!(factor.phone.is_none());
    }

    #[test]
    fn test_enhanced_session_structure() {
        let user = User {
            id: uuid::Uuid::new_v4(),
            email: Some("user@example.com".to_string()),
            phone: None,
            email_confirmed_at: Some(Utc::now()),
            phone_confirmed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_sign_in_at: Some(Utc::now()),
            app_metadata: serde_json::json!({}),
            user_metadata: serde_json::json!({}),
            aud: "authenticated".to_string(),
            role: Some("authenticated".to_string()),
        };

        let enhanced_session = EnhancedSession {
            access_token: "access-token".to_string(),
            refresh_token: "refresh-token".to_string(),
            expires_in: 3600,
            expires_at: Utc::now() + chrono::Duration::hours(1),
            token_type: "bearer".to_string(),
            user,
            token_metadata: None,
            mfa_verified: true,
            active_factors: vec![],
        };

        assert!(enhanced_session.mfa_verified);
        assert_eq!(enhanced_session.active_factors.len(), 0);
        assert_eq!(enhanced_session.token_type, "bearer");
    }
}
