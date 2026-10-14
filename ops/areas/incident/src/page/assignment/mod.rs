//! The page assignment records of incident's page work, and the rules, stores, and calculations around them.

mod page_assignment;
mod page_assignment_rules;
mod page_assignment_store;
mod page_assignment_engine;
mod page_assignment_view;
mod page_agreement;
mod page_agreement_rules;
mod page_agreement_store;
mod page_agreement_engine;
mod page_agreement_view;
mod page_agreement_report;
mod page_assignment_policy;
mod page_agreement_policy;
mod page_assignment_from_ledger;

pub use page_assignment::*;
pub use page_assignment_rules::*;
pub use page_assignment_store::*;
pub use page_assignment_engine::*;
pub use page_assignment_view::*;
pub use page_agreement::*;
pub use page_agreement_rules::*;
pub use page_agreement_store::*;
pub use page_agreement_engine::*;
pub use page_agreement_view::*;
pub use page_agreement_report::*;
pub use page_assignment_policy::*;
pub use page_agreement_policy::*;
pub use page_assignment_from_ledger::*;
