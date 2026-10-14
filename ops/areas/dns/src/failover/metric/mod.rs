//! The failover metric records of dns's failover work, and the rules, stores, and calculations around them.

mod failover_metric;
mod failover_metric_rules;
mod failover_metric_store;
mod failover_metric_report;
mod failover_metric_workflow;
mod failover_ledger;
mod failover_ledger_rules;
mod failover_ledger_view;
mod failover_allocation;
mod failover_allocation_rules;
mod failover_allocation_store;
mod failover_allocation_view;
mod failover_allocation_workflow;
mod failover_metric_policy;
mod failover_ledger_policy;
mod failover_allocation_policy;
mod failover_allocation_from_target;

pub use failover_metric::*;
pub use failover_metric_rules::*;
pub use failover_metric_store::*;
pub use failover_metric_report::*;
pub use failover_metric_workflow::*;
pub use failover_ledger::*;
pub use failover_ledger_rules::*;
pub use failover_ledger_view::*;
pub use failover_allocation::*;
pub use failover_allocation_rules::*;
pub use failover_allocation_store::*;
pub use failover_allocation_view::*;
pub use failover_allocation_workflow::*;
pub use failover_metric_policy::*;
pub use failover_ledger_policy::*;
pub use failover_allocation_policy::*;
pub use failover_allocation_from_target::*;
