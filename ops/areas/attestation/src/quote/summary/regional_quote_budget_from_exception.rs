use super::*;

use std::collections::BTreeMap;

use crate::policy::threshold as policy_threshold;

pub fn summarize_batched_policy_exceptions_for_budget(
    items: &[policy_threshold::BatchedPolicyException],
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

pub fn derive_regional_quote_budgets_from_exception(
    items: &[policy_threshold::BatchedPolicyException],
) -> Vec<RegionalQuoteBudget> {
    let mut out: Vec<RegionalQuoteBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_quote_budget(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_regional_quote_budget(item: &RegionalQuoteBudget) -> String {
    policy_threshold::normalize_batched_policy_exception_code(&item.code)
}
