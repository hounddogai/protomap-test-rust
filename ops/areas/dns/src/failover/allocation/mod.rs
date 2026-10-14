//! The scheduled failover allocation records of dns's failover work, and the rules, stores, and calculations
//! around them.

mod scheduled_failover_allocation;
mod scheduled_failover_allocation_rules;
mod scheduled_failover_allocation_view;
mod scheduled_failover_allocation_report;
mod failover_draft;
mod failover_draft_rules;
mod failover_draft_store;
mod failover_draft_engine;
mod failover_draft_view;
mod failover_draft_workflow;
mod scheduled_failover_allocation_policy;
mod failover_draft_policy;
mod failover_draft_from_variance;
mod scheduled_failover_allocation_from_policy;

pub use scheduled_failover_allocation::*;
pub use scheduled_failover_allocation_rules::*;
pub use scheduled_failover_allocation_view::*;
pub use scheduled_failover_allocation_report::*;
pub use failover_draft::*;
pub use failover_draft_rules::*;
pub use failover_draft_store::*;
pub use failover_draft_engine::*;
pub use failover_draft_view::*;
pub use failover_draft_workflow::*;
pub use scheduled_failover_allocation_policy::*;
pub use failover_draft_policy::*;
pub use failover_draft_from_variance::*;
pub use scheduled_failover_allocation_from_policy::*;
