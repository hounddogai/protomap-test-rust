//! The page rule records of incident's page work, and the rules, stores, and calculations around them.

mod page_rule;
mod page_rule_rules;
mod page_rule_store;
mod page_rule_view;
mod page_rule_report;
mod page_profile;
mod page_profile_view;
mod page_template;
mod page_template_rules;
mod page_template_store;
mod page_template_view;
mod page_template_report;
mod page_template_workflow;
mod page_rule_policy;
mod page_profile_policy;
mod page_template_policy;

pub use page_rule::*;
pub use page_rule_rules::*;
pub use page_rule_store::*;
pub use page_rule_view::*;
pub use page_rule_report::*;
pub use page_profile::*;
pub use page_profile_view::*;
pub use page_template::*;
pub use page_template_rules::*;
pub use page_template_store::*;
pub use page_template_view::*;
pub use page_template_report::*;
pub use page_template_workflow::*;
pub use page_rule_policy::*;
pub use page_profile_policy::*;
pub use page_template_policy::*;
