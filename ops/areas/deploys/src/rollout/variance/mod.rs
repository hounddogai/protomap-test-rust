//! The rollout variance records of deploys's rollout work, and the rules, stores, and calculations around
//! them.

mod rollout_variance;
mod rollout_variance_rules;
mod rollout_variance_store;
mod rollout_variance_report;
mod rollout_variance_workflow;
mod fallback_rollout_slot;
mod fallback_rollout_slot_store;
mod fallback_rollout_slot_report;
mod rollout_variance_policy;
mod fallback_rollout_slot_policy;
mod rollout_variance_from_record;

pub use rollout_variance::*;
pub use rollout_variance_rules::*;
pub use rollout_variance_store::*;
pub use rollout_variance_report::*;
pub use rollout_variance_workflow::*;
pub use fallback_rollout_slot::*;
pub use fallback_rollout_slot_store::*;
pub use fallback_rollout_slot_report::*;
pub use rollout_variance_policy::*;
pub use fallback_rollout_slot_policy::*;
pub use rollout_variance_from_record::*;
