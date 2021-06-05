/// Public modules
pub mod client;
pub mod error;
pub mod http;
pub mod models;

// Re-exports
pub use client::Client;

pub use error::Result;
