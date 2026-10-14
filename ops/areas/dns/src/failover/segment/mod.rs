//! The weighted failover segment records of dns's failover work, and the rules, stores, and calculations
//! around them.

mod weighted_failover_segment;
mod weighted_failover_segment_rules;
mod weighted_failover_segment_engine;
mod weighted_failover_segment_view;
mod failover_proposal;
mod failover_proposal_rules;
mod failover_proposal_engine;
mod failover_proposal_view;
mod failover_proposal_report;
mod weighted_failover_segment_policy;
mod failover_proposal_policy;
mod failover_proposal_from_group;

pub use weighted_failover_segment::*;
pub use weighted_failover_segment_rules::*;
pub use weighted_failover_segment_engine::*;
pub use weighted_failover_segment_view::*;
pub use failover_proposal::*;
pub use failover_proposal_rules::*;
pub use failover_proposal_engine::*;
pub use failover_proposal_view::*;
pub use failover_proposal_report::*;
pub use weighted_failover_segment_policy::*;
pub use failover_proposal_policy::*;
pub use failover_proposal_from_group::*;
