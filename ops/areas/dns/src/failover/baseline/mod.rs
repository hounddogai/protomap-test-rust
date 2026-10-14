//! The failover baseline records of dns's failover work, and the rules, stores, and calculations around them.

mod failover_baseline;
mod failover_baseline_rules;
mod failover_baseline_store;
mod failover_baseline_view;
mod failover_baseline_workflow;
mod failover_group;
mod failover_group_rules;
mod failover_review;
mod failover_review_rules;
mod failover_review_engine;
mod failover_review_view;
mod failover_review_report;
mod failover_baseline_policy;
mod failover_group_policy;
mod failover_review_policy;
mod failover_group_from_schedule;
mod failover_group_from_allocation;

pub use failover_baseline::*;
pub use failover_baseline_rules::*;
pub use failover_baseline_store::*;
pub use failover_baseline_view::*;
pub use failover_baseline_workflow::*;
pub use failover_group::*;
pub use failover_group_rules::*;
pub use failover_review::*;
pub use failover_review_rules::*;
pub use failover_review_engine::*;
pub use failover_review_view::*;
pub use failover_review_report::*;
pub use failover_baseline_policy::*;
pub use failover_group_policy::*;
pub use failover_review_policy::*;
pub use failover_group_from_schedule::*;
pub use failover_group_from_allocation::*;
