//! The adaptive failover exception records of dns's failover work, and the rules, stores, and calculations
//! around them.

mod adaptive_failover_exception;
mod adaptive_failover_exception_rules;
mod adaptive_failover_exception_store;
mod adaptive_failover_exception_engine;
mod adaptive_failover_exception_workflow;
mod pending_failover_segment;
mod pending_failover_segment_rules;
mod adaptive_failover_exception_policy;
mod pending_failover_segment_policy;
mod adaptive_failover_exception_from_summary;

pub use adaptive_failover_exception::*;
pub use adaptive_failover_exception_rules::*;
pub use adaptive_failover_exception_store::*;
pub use adaptive_failover_exception_engine::*;
pub use adaptive_failover_exception_workflow::*;
pub use pending_failover_segment::*;
pub use pending_failover_segment_rules::*;
pub use adaptive_failover_exception_policy::*;
pub use pending_failover_segment_policy::*;
pub use adaptive_failover_exception_from_summary::*;
