//! Taiwan Stock Exchange (TWSE) API
//!
//! Example:
//! ```rust
//! use twstock::*;
//!
//! async fn fetch() {
//!     let client = Client::new();
//!     let data = client.realtime().fetch(Stock::Live(2330)).await.unwrap();
//!     assert_eq!(data.name, "台積電");
//! }
//! ```
//!
//! Features:
//! - `serde`: Enable serde support
//! - `native-tls`: Use the native-tls backend
//! - `native-tls-vendored`: Use the native-tls backend with vendored OpenSSL
//! - `rustls-tls`: Use the rustls backend
//!
//! Don't forget to disable default features if you want to use a specific TLS backend.

pub mod history;
pub mod realtime;

use reqwest::Client as HttpClient;

fn get_time_zone() -> chrono::FixedOffset {
    chrono::FixedOffset::east_opt(8 * 3600).unwrap()
}

/// Error type that may occur when interacting with the TWSE API
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("incompatible upstream api")]
    IncompatibleApi,
    #[error("date does not exist")]
    DateDoesNotExist,
    #[error("Error message from upstream: {0}")]
    ErrorStatMessage(String),
    #[error("market is closed")]
    MarketClosed,
}

/// Stock identifier and its variant
pub enum Stock {
    /// Live stock
    Live(u32),
    /// Over-the-counter stock
    OverTheCounter(u32),
}

impl Stock {
    /// Get the stock identifier
    pub fn id(&self) -> u32 {
        match self {
            Stock::Live(id) => *id,
            Stock::OverTheCounter(id) => *id,
        }
    }
}

/// Client for fetching data from the Taiwan Stock Exchange (TWSE) API
#[derive(Default)]
pub struct Client(HttpClient);

impl Client {
    /// Create a new client
    pub fn new() -> Self {
        Self::default()
    }
}

// if not TLS feature enabled, compile error
#[cfg(not(any(
    feature = "default-tls",
    feature = "native-tls",
    feature = "native-tls-vendored",
    feature = "rustls-tls"
)))]
compile_error!("TLS feature is not enabled");
