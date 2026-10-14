use super::*;

use std::collections::BTreeMap;

use crate::canary::entry as canary_entry;

pub fn summarize_primary_canary_quotas_for_budget(items: &[canary_entry::PrimaryCanaryQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_freeze_budgets_from_quota(items: &[canary_entry::PrimaryCanaryQuota]) -> Vec<FreezeBudget> {
    let mut out: Vec<FreezeBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_budget(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_freeze_budget(item: &FreezeBudget) -> String {
    canary_entry::normalize_regional_canary_summary_code(&item.code)
}
