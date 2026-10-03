//! Google APIs: OAuth sign-in and the Cloud Code quota endpoints.

pub mod cloudcode;
pub mod loopback;
pub mod oauth;

/// User-Agent for every request. Cloud Code picks the product line from the
/// User-Agent and answers 403 ("no valid license") unless it mentions
/// antigravity, so AgyOrbit names itself and states which product it serves.
pub const USER_AGENT: &str = concat!("AgyOrbit/", env!("CARGO_PKG_VERSION"), " (antigravity)");
