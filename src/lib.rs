//! # Taiwan Stock Exchange (TWSE) API
//!
//! `twstock` is a library for fetching data from the Taiwan Stock Exchange (TWSE) API.
//! 
//! Behind the scenes, it uses the reqwest crate to make HTTP requests to their server.
//! 
//! # Example:
//! ```rust
//! use twstock::*;
//!
//! async fn fetch() {
//!     let client = Client::new();
//!     match client
//!         .realtime()
//!         .fetch(Stock {
//!             kind: StockKind::Live,
//!             code: 2330,
//!         })
//!         .await
//!     {
//!         Ok(x) => assert_eq!(x.name, "台積電"),
//!         Err(err) => match err {
//!             Error::MarketClosed => {}
//!             _ => panic!("unexpected error: {:?}", err),
//!         },
//!     };
//! }
//! ```
//!
//! # Features:
//! - `serde`: Enable serde support
//! - `native-tls`: Use the native-tls backend
//! - `native-tls-vendored`: Use the native-tls backend with vendored OpenSSL
//! - `rustls-tls`: Use the rustls backend
//!
//! Don't forget to disable default features if you want to use a specific TLS backend.

pub mod history;
pub mod list;
pub mod realtime;

use reqwest::Client as HttpClient;

fn get_time_zone() -> chrono::FixedOffset {
    chrono::FixedOffset::east_opt(8 * 3600).unwrap()
}

/// Error type that may occur when interacting with the TWSE API
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    /// Incompatible API, the upstream API has changed
    #[error("incompatible upstream api")]
    IncompatibleApi,
    #[error("date does not exist")]
    DateDoesNotExist,
    #[error("Error message from upstream: `{0}`")]
    StatMessage(String),
    #[error("market is closed")]
    MarketClosed,
}

#[derive(Debug, Hash, Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Stock identifier and its variant
pub struct Stock {
    pub kind: StockKind,
    pub code: u32,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
/// variant of stock
pub enum StockKind {
    #[default]
    Live = 2,
    OverTheCounter = 4,
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
