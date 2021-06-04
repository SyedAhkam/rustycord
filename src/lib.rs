/// Public modules
pub mod client;
pub mod errors;
pub mod http;
pub mod models;

/// Re-exports
pub use client::Client;

pub use errors::RustyCordError;
pub use errors::RustyCordResult;
