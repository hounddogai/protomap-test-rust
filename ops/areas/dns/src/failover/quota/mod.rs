//! The failover quota records of dns's failover work, and the rules, stores, and calculations around them.

mod failover_quota;
mod failover_quota_rules;
mod failover_quota_view;
mod failover_snapshot;
mod failover_snapshot_rules;
mod failover_snapshot_store;
mod failover_snapshot_engine;
mod failover_snapshot_workflow;
mod failover_rule;
mod failover_rule_rules;
mod failover_rule_engine;
mod failover_rule_report;
mod failover_quota_policy;
mod failover_snapshot_policy;
mod failover_rule_policy;
mod failover_snapshot_from_quota;
mod failover_snapshot_from_summary;

pub use failover_quota::*;
pub use failover_quota_rules::*;
pub use failover_quota_view::*;
pub use failover_snapshot::*;
pub use failover_snapshot_rules::*;
pub use failover_snapshot_store::*;
pub use failover_snapshot_engine::*;
pub use failover_snapshot_workflow::*;
pub use failover_rule::*;
pub use failover_rule_rules::*;
pub use failover_rule_engine::*;
pub use failover_rule_report::*;
pub use failover_quota_policy::*;
pub use failover_snapshot_policy::*;
pub use failover_rule_policy::*;
pub use failover_snapshot_from_quota::*;
pub use failover_snapshot_from_summary::*;
