//! The page allocation records of incident's page work, and the rules, stores, and calculations around them.

mod page_allocation;
mod page_allocation_rules;
mod page_allocation_store;
mod page_allocation_report;
mod capped_page_summary;
mod capped_page_summary_rules;
mod capped_page_summary_store;
mod capped_page_summary_view;
mod capped_page_summary_workflow;
mod page_allocation_policy;
mod capped_page_summary_policy;
mod capped_page_summary_from_ledger;

pub use page_allocation::*;
pub use page_allocation_rules::*;
pub use page_allocation_store::*;
pub use page_allocation_report::*;
pub use capped_page_summary::*;
pub use capped_page_summary_rules::*;
pub use capped_page_summary_store::*;
pub use capped_page_summary_view::*;
pub use capped_page_summary_workflow::*;
pub use page_allocation_policy::*;
pub use capped_page_summary_policy::*;
pub use capped_page_summary_from_ledger::*;
