//! Daemon client module for gRPC communication

mod client;
mod traits;

pub use client::DaemonClient;

#[cfg(test)]
pub use traits::{DaemonClientTrait, MockDaemonClientTrait};
