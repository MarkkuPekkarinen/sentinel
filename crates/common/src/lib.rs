//! Common utilities and shared components for Sentinel proxy
//!
//! This crate provides shared functionality used across all Sentinel components,
//! including observability (metrics, logging, tracing), error types, and common utilities.

pub mod observability;

pub mod errors;
pub mod limits;
pub mod types;

// Re-export commonly used items at the crate root
pub use observability::{
    init_tracing, AuditLogEntry, ComponentHealth, HealthChecker, HealthStatus, RequestMetrics,
};

// Re-export error types
pub use errors::{SentinelError, SentinelResult};

// Re-export limit types
pub use limits::{Limits, RateLimiter};

// Re-export common types
pub use types::{CorrelationId, RequestId};
