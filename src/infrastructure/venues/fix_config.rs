//! # FIX Configuration
//!
//! Configuration for FIX sessions.
//!
//! This module provides configuration types for FIX protocol sessions
//! including session parameters, TLS settings, and timeout configuration.
//!
//! # Examples
//!
//! ```
//! use otc_rfq::infrastructure::venues::fix_config::FixSessionConfig;
//!
//! let config = FixSessionConfig::new("OTC_PLATFORM", "MARKET_MAKER")
//!     .with_host("fix.example.com")
//!     .with_port(9876)
//!     .with_heartbeat_interval(30);
//! ```

use crate::domain::value_objects::VenueId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default heartbeat interval in seconds.
const DEFAULT_HEARTBEAT_INTERVAL: u32 = 30;

/// Default reconnect interval in seconds.
const DEFAULT_RECONNECT_INTERVAL: u32 = 5;

/// Default quote timeout in milliseconds.
const DEFAULT_QUOTE_TIMEOUT_MS: u64 = 5000;

/// Default execution timeout in milliseconds.
const DEFAULT_EXECUTION_TIMEOUT_MS: u64 = 10000;

/// Default FIX port.
const DEFAULT_FIX_PORT: u16 = 9876;

/// FIX protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FixVersion {
    /// FIX 4.0.
    Fix40,
    /// FIX 4.2.
    Fix42,
    /// FIX 4.4 (most common).
    #[default]
    Fix44,
    /// FIX 5.0 SP2.
    Fix50Sp2,
}

impl FixVersion {
    /// Returns the FIX version string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fix40 => "FIX.4.0",
            Self::Fix42 => "FIX.4.2",
            Self::Fix44 => "FIX.4.4",
            Self::Fix50Sp2 => "FIXT.1.1",
        }
    }
}

impl std::fmt::Display for FixVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// TLS configuration for FIX sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Whether TLS is enabled.
    enabled: bool,
    /// Path to the certificate file.
    certificate_path: Option<PathBuf>,
    /// Path to the private key file.
    private_key_path: Option<PathBuf>,
    /// Path to the CA certificate file.
    ca_certificate_path: Option<PathBuf>,
    /// Whether to verify the server certificate.
    verify_server: bool,
}

impl TlsConfig {
    /// Creates a new TLS configuration with TLS disabled.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            certificate_path: None,
            private_key_path: None,
            ca_certificate_path: None,
            verify_server: true,
        }
    }

    /// Creates a new TLS configuration with TLS enabled.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            certificate_path: None,
            private_key_path: None,
            ca_certificate_path: None,
            verify_server: true,
        }
    }

    /// Sets the certificate path.
    #[must_use]
    pub fn with_certificate(mut self, path: impl Into<PathBuf>) -> Self {
        self.certificate_path = Some(path.into());
        self
    }

    /// Sets the private key path.
    #[must_use]
    pub fn with_private_key(mut self, path: impl Into<PathBuf>) -> Self {
        self.private_key_path = Some(path.into());
        self
    }

    /// Sets the CA certificate path.
    #[must_use]
    pub fn with_ca_certificate(mut self, path: impl Into<PathBuf>) -> Self {
        self.ca_certificate_path = Some(path.into());
        self
    }

    /// Sets whether to verify the server certificate.
    #[must_use]
    pub fn with_verify_server(mut self, verify: bool) -> Self {
        self.verify_server = verify;
        self
    }

    /// Returns whether TLS is enabled.
    #[inline]
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the certificate path.
    #[inline]
    #[must_use]
    pub fn certificate_path(&self) -> Option<&PathBuf> {
        self.certificate_path.as_ref()
    }

    /// Returns the private key path.
    #[inline]
    #[must_use]
    pub fn private_key_path(&self) -> Option<&PathBuf> {
        self.private_key_path.as_ref()
    }

    /// Returns the CA certificate path.
    #[inline]
    #[must_use]
    pub fn ca_certificate_path(&self) -> Option<&PathBuf> {
        self.ca_certificate_path.as_ref()
    }

    /// Returns whether to verify the server certificate.
    #[inline]
    #[must_use]
    pub fn verify_server(&self) -> bool {
        self.verify_server
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self::disabled()
    }
}

/// Logon credentials for FIX sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogonCredentials {
    /// Username for logon.
    username: Option<String>,
    /// Password for logon (should be loaded from environment).
    password: Option<String>,
    /// Environment variable name for password.
    password_env: Option<String>,
}

impl LogonCredentials {
    /// Creates empty credentials.
    #[must_use]
    pub fn none() -> Self {
        Self {
            username: None,
            password: None,
            password_env: None,
        }
    }

    /// Creates credentials with username and password.
    #[must_use]
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: Some(username.into()),
            password: Some(password.into()),
            password_env: None,
        }
    }

    /// Creates credentials with username and password from environment.
    #[must_use]
    pub fn from_env(username: impl Into<String>, password_env: impl Into<String>) -> Self {
        Self {
            username: Some(username.into()),
            password: None,
            password_env: Some(password_env.into()),
        }
    }

    /// Returns the username.
    #[inline]
    #[must_use]
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// Returns the password, loading from environment if needed.
    #[must_use]
    pub fn password(&self) -> Option<String> {
        if let Some(ref pwd) = self.password {
            return Some(pwd.clone());
        }
        if let Some(ref env_var) = self.password_env {
            return std::env::var(env_var).ok();
        }
        None
    }

    /// Returns true if credentials are configured.
    #[must_use]
    pub fn has_credentials(&self) -> bool {
        self.username.is_some()
    }
}

impl Default for LogonCredentials {
    fn default() -> Self {
        Self::none()
    }
}

/// FIX session configuration.
///
/// Contains all parameters needed to establish and maintain a FIX session.
///
/// # Examples
///
/// ```
/// use otc_rfq::infrastructure::venues::fix_config::FixSessionConfig;
///
/// let config = FixSessionConfig::new("OTC_PLATFORM", "MARKET_MAKER")
///     .with_host("fix.example.com")
///     .with_port(9876)
///     .with_heartbeat_interval(30);
///
/// assert_eq!(config.sender_comp_id(), "OTC_PLATFORM");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixSessionConfig {
    /// Sender CompID (our identifier).
    sender_comp_id: String,
    /// Target CompID (counterparty identifier).
    target_comp_id: String,
    /// FIX protocol version.
    fix_version: FixVersion,
    /// Host address.
    host: String,
    /// Port number.
    port: u16,
    /// Heartbeat interval in seconds.
    heartbeat_interval: u32,
    /// Reconnect interval in seconds.
    reconnect_interval: u32,
    /// Whether to reset sequence numbers on logon.
    reset_on_logon: bool,
    /// Whether to reset sequence numbers on logout.
    reset_on_logout: bool,
    /// Whether to reset sequence numbers on disconnect.
    reset_on_disconnect: bool,
    /// TLS configuration.
    tls: TlsConfig,
    /// Logon credentials.
    credentials: LogonCredentials,
    /// Path to data dictionary file.
    data_dictionary_path: Option<PathBuf>,
}

impl FixSessionConfig {
    /// Creates a new FIX session configuration.
    #[must_use]
    pub fn new(sender_comp_id: impl Into<String>, target_comp_id: impl Into<String>) -> Self {
        Self {
            sender_comp_id: sender_comp_id.into(),
            target_comp_id: target_comp_id.into(),
            fix_version: FixVersion::default(),
            host: String::new(),
            port: DEFAULT_FIX_PORT,
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL,
            reconnect_interval: DEFAULT_RECONNECT_INTERVAL,
            reset_on_logon: true,
            reset_on_logout: true,
            reset_on_disconnect: true,
            tls: TlsConfig::default(),
            credentials: LogonCredentials::default(),
            data_dictionary_path: None,
        }
    }

    /// Sets the FIX version.
    #[must_use]
    pub fn with_fix_version(mut self, version: FixVersion) -> Self {
        self.fix_version = version;
        self
    }

    /// Sets the host address.
    #[must_use]
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Sets the port number.
    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the heartbeat interval in seconds.
    #[must_use]
    pub fn with_heartbeat_interval(mut self, interval: u32) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Sets the reconnect interval in seconds.
    #[must_use]
    pub fn with_reconnect_interval(mut self, interval: u32) -> Self {
        self.reconnect_interval = interval;
        self
    }

    /// Sets whether to reset sequence numbers on logon.
    #[must_use]
    pub fn with_reset_on_logon(mut self, reset: bool) -> Self {
        self.reset_on_logon = reset;
        self
    }

    /// Sets whether to reset sequence numbers on logout.
    #[must_use]
    pub fn with_reset_on_logout(mut self, reset: bool) -> Self {
        self.reset_on_logout = reset;
        self
    }

    /// Sets whether to reset sequence numbers on disconnect.
    #[must_use]
    pub fn with_reset_on_disconnect(mut self, reset: bool) -> Self {
        self.reset_on_disconnect = reset;
        self
    }

    /// Sets the TLS configuration.
    #[must_use]
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls = tls;
        self
    }

    /// Sets the logon credentials.
    #[must_use]
    pub fn with_credentials(mut self, credentials: LogonCredentials) -> Self {
        self.credentials = credentials;
        self
    }

    /// Sets the data dictionary path.
    #[must_use]
    pub fn with_data_dictionary(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dictionary_path = Some(path.into());
        self
    }

    /// Returns the sender CompID.
    #[inline]
    #[must_use]
    pub fn sender_comp_id(&self) -> &str {
        &self.sender_comp_id
    }

    /// Returns the target CompID.
    #[inline]
    #[must_use]
    pub fn target_comp_id(&self) -> &str {
        &self.target_comp_id
    }

    /// Returns the FIX version.
    #[inline]
    #[must_use]
    pub fn fix_version(&self) -> FixVersion {
        self.fix_version
    }

    /// Returns the host address.
    #[inline]
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the port number.
    #[inline]
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the heartbeat interval in seconds.
    #[inline]
    #[must_use]
    pub fn heartbeat_interval(&self) -> u32 {
        self.heartbeat_interval
    }

    /// Returns the reconnect interval in seconds.
    #[inline]
    #[must_use]
    pub fn reconnect_interval(&self) -> u32 {
        self.reconnect_interval
    }

    /// Returns whether to reset sequence numbers on logon.
    #[inline]
    #[must_use]
    pub fn reset_on_logon(&self) -> bool {
        self.reset_on_logon
    }

    /// Returns whether to reset sequence numbers on logout.
    #[inline]
    #[must_use]
    pub fn reset_on_logout(&self) -> bool {
        self.reset_on_logout
    }

    /// Returns whether to reset sequence numbers on disconnect.
    #[inline]
    #[must_use]
    pub fn reset_on_disconnect(&self) -> bool {
        self.reset_on_disconnect
    }

    /// Returns the TLS configuration.
    #[inline]
    #[must_use]
    pub fn tls(&self) -> &TlsConfig {
        &self.tls
    }

    /// Returns the logon credentials.
    #[inline]
    #[must_use]
    pub fn credentials(&self) -> &LogonCredentials {
        &self.credentials
    }

    /// Returns the data dictionary path.
    #[inline]
    #[must_use]
    pub fn data_dictionary_path(&self) -> Option<&PathBuf> {
        self.data_dictionary_path.as_ref()
    }

    /// Returns the connection address as "host:port".
    #[must_use]
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// FIX Market Maker adapter configuration.
///
/// Contains session configuration plus adapter-specific settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixMMConfig {
    /// Venue ID for this adapter.
    venue_id: VenueId,
    /// FIX session configuration.
    session: FixSessionConfig,
    /// Quote request timeout in milliseconds.
    quote_timeout_ms: u64,
    /// Execution timeout in milliseconds.
    execution_timeout_ms: u64,
    /// Whether the adapter is enabled.
    enabled: bool,
}

impl FixMMConfig {
    /// Creates a new FIX MM configuration.
    #[must_use]
    pub fn new(venue_id: impl Into<String>, session: FixSessionConfig) -> Self {
        Self {
            venue_id: VenueId::new(venue_id),
            session,
            quote_timeout_ms: DEFAULT_QUOTE_TIMEOUT_MS,
            execution_timeout_ms: DEFAULT_EXECUTION_TIMEOUT_MS,
            enabled: true,
        }
    }

    /// Sets the quote timeout in milliseconds.
    #[must_use]
    pub fn with_quote_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.quote_timeout_ms = timeout_ms;
        self
    }

    /// Sets the execution timeout in milliseconds.
    #[must_use]
    pub fn with_execution_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.execution_timeout_ms = timeout_ms;
        self
    }

    /// Sets whether the adapter is enabled.
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Returns the venue ID.
    #[inline]
    #[must_use]
    pub fn venue_id(&self) -> &VenueId {
        &self.venue_id
    }

    /// Returns the session configuration.
    #[inline]
    #[must_use]
    pub fn session(&self) -> &FixSessionConfig {
        &self.session
    }

    /// Returns the quote timeout in milliseconds.
    #[inline]
    #[must_use]
    pub fn quote_timeout_ms(&self) -> u64 {
        self.quote_timeout_ms
    }

    /// Returns the execution timeout in milliseconds.
    #[inline]
    #[must_use]
    pub fn execution_timeout_ms(&self) -> u64 {
        self.execution_timeout_ms
    }

    /// Returns whether the adapter is enabled.
    #[inline]
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fix_version {
        use super::*;

        #[test]
        fn as_str() {
            assert_eq!(FixVersion::Fix40.as_str(), "FIX.4.0");
            assert_eq!(FixVersion::Fix42.as_str(), "FIX.4.2");
            assert_eq!(FixVersion::Fix44.as_str(), "FIX.4.4");
            assert_eq!(FixVersion::Fix50Sp2.as_str(), "FIXT.1.1");
        }

        #[test]
        fn default_is_fix44() {
            assert_eq!(FixVersion::default(), FixVersion::Fix44);
        }

        #[test]
        fn display() {
            assert_eq!(FixVersion::Fix44.to_string(), "FIX.4.4");
        }
    }

    mod tls_config {
        use super::*;

        #[test]
        fn disabled_by_default() {
            let config = TlsConfig::default();
            assert!(!config.is_enabled());
        }

        #[test]
        fn enabled() {
            let config = TlsConfig::enabled();
            assert!(config.is_enabled());
        }

        #[test]
        fn with_certificate() {
            let config = TlsConfig::enabled().with_certificate("/path/to/cert.pem");
            assert_eq!(
                config.certificate_path(),
                Some(&PathBuf::from("/path/to/cert.pem"))
            );
        }
    }

    mod logon_credentials {
        use super::*;

        #[test]
        fn none() {
            let creds = LogonCredentials::none();
            assert!(!creds.has_credentials());
            assert!(creds.username().is_none());
        }

        #[test]
        fn with_password() {
            let creds = LogonCredentials::new("user", "pass");
            assert!(creds.has_credentials());
            assert_eq!(creds.username(), Some("user"));
            assert_eq!(creds.password(), Some("pass".to_string()));
        }
    }

    mod fix_session_config {
        use super::*;

        #[test]
        fn new() {
            let config = FixSessionConfig::new("SENDER", "TARGET");
            assert_eq!(config.sender_comp_id(), "SENDER");
            assert_eq!(config.target_comp_id(), "TARGET");
            assert_eq!(config.fix_version(), FixVersion::Fix44);
        }

        #[test]
        fn with_host_and_port() {
            let config = FixSessionConfig::new("SENDER", "TARGET")
                .with_host("fix.example.com")
                .with_port(9999);
            assert_eq!(config.host(), "fix.example.com");
            assert_eq!(config.port(), 9999);
            assert_eq!(config.address(), "fix.example.com:9999");
        }

        #[test]
        fn with_heartbeat() {
            let config = FixSessionConfig::new("SENDER", "TARGET").with_heartbeat_interval(60);
            assert_eq!(config.heartbeat_interval(), 60);
        }

        #[test]
        fn with_tls() {
            let config = FixSessionConfig::new("SENDER", "TARGET").with_tls(TlsConfig::enabled());
            assert!(config.tls().is_enabled());
        }
    }

    mod fix_mm_config {
        use super::*;

        #[test]
        fn new() {
            let session = FixSessionConfig::new("SENDER", "TARGET");
            let config = FixMMConfig::new("fix-mm-1", session);

            assert_eq!(config.venue_id(), &VenueId::new("fix-mm-1"));
            assert!(config.is_enabled());
        }

        #[test]
        fn with_timeouts() {
            let session = FixSessionConfig::new("SENDER", "TARGET");
            let config = FixMMConfig::new("fix-mm-1", session)
                .with_quote_timeout_ms(3000)
                .with_execution_timeout_ms(8000);

            assert_eq!(config.quote_timeout_ms(), 3000);
            assert_eq!(config.execution_timeout_ms(), 8000);
        }

        #[test]
        fn disabled() {
            let session = FixSessionConfig::new("SENDER", "TARGET");
            let config = FixMMConfig::new("fix-mm-1", session).with_enabled(false);
            assert!(!config.is_enabled());
        }
    }
}
