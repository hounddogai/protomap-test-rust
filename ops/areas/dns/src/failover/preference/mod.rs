//! The failover preference records of dns's failover work, and the rules, stores, and calculations around
//! them.

mod failover_preference;
mod failover_preference_rules;
mod failover_preference_engine;
mod failover_reservation;
mod failover_reservation_store;
mod failover_reservation_engine;
mod failover_reservation_report;
mod failover_milestone;
mod failover_milestone_rules;
mod failover_milestone_store;
mod failover_milestone_workflow;
mod failover_preference_policy;
mod failover_reservation_policy;
mod failover_milestone_policy;
mod failover_milestone_from_reservation;

pub use failover_preference::*;
pub use failover_preference_rules::*;
pub use failover_preference_engine::*;
pub use failover_reservation::*;
pub use failover_reservation_store::*;
pub use failover_reservation_engine::*;
pub use failover_reservation_report::*;
pub use failover_milestone::*;
pub use failover_milestone_rules::*;
pub use failover_milestone_store::*;
pub use failover_milestone_workflow::*;
pub use failover_preference_policy::*;
pub use failover_reservation_policy::*;
pub use failover_milestone_policy::*;
pub use failover_milestone_from_reservation::*;
