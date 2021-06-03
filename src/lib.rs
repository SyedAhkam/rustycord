/// Public modules
pub mod client;
pub mod errors;
pub mod http;

/// Re-exports
pub use client::Client;

pub use errors::RustyCordError;
pub use errors::RustyCordResult;
