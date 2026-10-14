//! The default canary allocation records of deploys's canary work, and the rules, stores, and calculations
//! around them.

mod default_canary_allocation;
mod default_canary_allocation_rules;
mod default_canary_allocation_store;
mod default_canary_allocation_engine;
mod seasonal_canary_threshold;
mod seasonal_canary_threshold_rules;
mod seasonal_canary_threshold_store;
mod seasonal_canary_threshold_engine;
mod seasonal_canary_threshold_view;
mod regional_canary_agreement;
mod regional_canary_agreement_rules;
mod regional_canary_agreement_engine;
mod regional_canary_agreement_view;
mod legacy_canary_agreement;
mod legacy_canary_agreement_rules;
mod legacy_canary_agreement_engine;
mod default_canary_allocation_policy;
mod seasonal_canary_threshold_policy;
mod regional_canary_agreement_policy;
mod legacy_canary_agreement_policy;
mod default_canary_allocation_from_target;

pub use default_canary_allocation::*;
pub use default_canary_allocation_rules::*;
pub use default_canary_allocation_store::*;
pub use default_canary_allocation_engine::*;
pub use seasonal_canary_threshold::*;
pub use seasonal_canary_threshold_rules::*;
pub use seasonal_canary_threshold_store::*;
pub use seasonal_canary_threshold_engine::*;
pub use seasonal_canary_threshold_view::*;
pub use regional_canary_agreement::*;
pub use regional_canary_agreement_rules::*;
pub use regional_canary_agreement_engine::*;
pub use regional_canary_agreement_view::*;
pub use legacy_canary_agreement::*;
pub use legacy_canary_agreement_rules::*;
pub use legacy_canary_agreement_engine::*;
pub use default_canary_allocation_policy::*;
pub use seasonal_canary_threshold_policy::*;
pub use regional_canary_agreement_policy::*;
pub use legacy_canary_agreement_policy::*;
pub use default_canary_allocation_from_target::*;
