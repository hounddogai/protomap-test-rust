//! The dns record template records of dns's dns record work, and the rules, stores, and calculations around
//! them.

mod dns_record_template;
mod dns_record_template_rules;
mod dns_record_template_store;
mod dns_record_template_view;
mod dns_record_reservation;
mod dns_record_reservation_rules;
mod dns_record_reservation_store;
mod dns_record_reservation_engine;
mod dns_record_reservation_view;
mod dns_record_reservation_workflow;
mod dns_record_quota;
mod dns_record_quota_rules;
mod dns_record_quota_engine;
mod dns_record_quota_report;
mod tiered_dns_record_policy;
mod tiered_dns_record_policy_rules;
mod tiered_dns_record_policy_engine;
mod dns_record_template_policy;
mod dns_record_reservation_policy;
mod dns_record_quota_policy;
mod tiered_dns_record_policy_policy;
mod tiered_dns_record_policy_from_review;

pub use dns_record_template::*;
pub use dns_record_template_rules::*;
pub use dns_record_template_store::*;
pub use dns_record_template_view::*;
pub use dns_record_reservation::*;
pub use dns_record_reservation_rules::*;
pub use dns_record_reservation_store::*;
pub use dns_record_reservation_engine::*;
pub use dns_record_reservation_view::*;
pub use dns_record_reservation_workflow::*;
pub use dns_record_quota::*;
pub use dns_record_quota_rules::*;
pub use dns_record_quota_engine::*;
pub use dns_record_quota_report::*;
pub use tiered_dns_record_policy::*;
pub use tiered_dns_record_policy_rules::*;
pub use tiered_dns_record_policy_engine::*;
pub use dns_record_template_policy::*;
pub use dns_record_reservation_policy::*;
pub use dns_record_quota_policy::*;
pub use tiered_dns_record_policy_policy::*;
pub use tiered_dns_record_policy_from_review::*;
