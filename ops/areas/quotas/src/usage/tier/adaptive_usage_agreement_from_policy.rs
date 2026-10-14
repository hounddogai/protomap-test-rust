use super::*;

use std::collections::BTreeMap;

use crate::limit::limit_policy as limit_limit_policy;

pub fn summarize_default_limit_policies_for_agreement(
    items: &[limit_limit_policy::DefaultLimitPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_adaptive_usage_agreements_from_policy(
    items: &[limit_limit_policy::DefaultLimitPolicy],
) -> Vec<AdaptiveUsageAgreement> {
    let mut out: Vec<AdaptiveUsageAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_usage_agreement(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_adaptive_usage_agreement(item: &AdaptiveUsageAgreement) -> String {
    limit_limit_policy::normalize_primary_limit_adjustment_code(&item.code)
}
