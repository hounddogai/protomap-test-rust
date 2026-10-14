//! The legacy failover adjustment records of dns's failover work, and the rules, stores, and calculations
//! around them.

mod legacy_failover_adjustment;
mod legacy_failover_adjustment_store;
mod legacy_failover_adjustment_engine;
mod legacy_failover_adjustment_view;
mod failover_bundle;
mod failover_bundle_store;
mod failover_bundle_report;
mod weighted_failover_group;
mod weighted_failover_group_rules;
mod weighted_failover_group_store;
mod weighted_failover_group_engine;
mod weighted_failover_group_view;
mod weighted_failover_group_workflow;
mod legacy_failover_adjustment_policy;
mod failover_bundle_policy;
mod weighted_failover_group_policy;
mod weighted_failover_group_from_override;
mod weighted_failover_group_from_ledger;

pub use legacy_failover_adjustment::*;
pub use legacy_failover_adjustment_store::*;
pub use legacy_failover_adjustment_engine::*;
pub use legacy_failover_adjustment_view::*;
pub use failover_bundle::*;
pub use failover_bundle_store::*;
pub use failover_bundle_report::*;
pub use weighted_failover_group::*;
pub use weighted_failover_group_rules::*;
pub use weighted_failover_group_store::*;
pub use weighted_failover_group_engine::*;
pub use weighted_failover_group_view::*;
pub use weighted_failover_group_workflow::*;
pub use legacy_failover_adjustment_policy::*;
pub use failover_bundle_policy::*;
pub use weighted_failover_group_policy::*;
pub use weighted_failover_group_from_override::*;
pub use weighted_failover_group_from_ledger::*;
