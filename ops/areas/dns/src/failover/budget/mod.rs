//! The failover budget records of dns's failover work, and the rules, stores, and calculations around them.

mod failover_budget;
mod failover_budget_rules;
mod failover_budget_view;
mod failover_budget_report;
mod batched_failover_review;
mod batched_failover_review_rules;
mod batched_failover_review_engine;
mod batched_failover_review_view;
mod failover_window;
mod failover_window_rules;
mod failover_window_engine;
mod failover_budget_policy;
mod batched_failover_review_policy;
mod failover_window_policy;
mod failover_window_from_group;
mod failover_window_from_estimate;

pub use failover_budget::*;
pub use failover_budget_rules::*;
pub use failover_budget_view::*;
pub use failover_budget_report::*;
pub use batched_failover_review::*;
pub use batched_failover_review_rules::*;
pub use batched_failover_review_engine::*;
pub use batched_failover_review_view::*;
pub use failover_window::*;
pub use failover_window_rules::*;
pub use failover_window_engine::*;
pub use failover_budget_policy::*;
pub use batched_failover_review_policy::*;
pub use failover_window_policy::*;
pub use failover_window_from_group::*;
pub use failover_window_from_estimate::*;
