use super::*;

use std::collections::BTreeMap;

use crate::approval::preference as approval_preference;

pub fn summarize_strict_approval_preferences_for_summary(
    items: &[approval_preference::StrictApprovalPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_rollback_summaries_from_preference(
    items: &[approval_preference::StrictApprovalPreference],
) -> Vec<RollbackSummary> {
    let mut out: Vec<RollbackSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_rollback_summary(item: &RollbackSummary) -> f64 {
    approval_preference::approval_group_weight(5, true)
}
