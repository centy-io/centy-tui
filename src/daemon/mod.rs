//! Daemon client module for gRPC communication

mod client;
mod traits;

pub use client::DaemonClient;

#[cfg(test)]
#[allow(unused_imports)]
pub use traits::{DaemonClientTrait, MockDaemonClientTrait};
