use super::*;

use std::collections::BTreeMap;

use crate::canary::policy as canary_policy;

pub fn summarize_canary_scores_for_entry(items: &[canary_policy::CanaryScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_fallback_approval_entries_from_score(items: &[canary_policy::CanaryScore]) -> Vec<FallbackApprovalEntry> {
    let mut out: Vec<FallbackApprovalEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_approval_entry(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_fallback_approval_entry(item: &FallbackApprovalEntry) -> String {
    canary_policy::classify_canary_ledger(item.projected_load())
}
