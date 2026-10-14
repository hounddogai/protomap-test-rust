use super::*;

use std::collections::BTreeMap;

use crate::approval::claim as approval_claim;

pub fn summarize_regional_approval_groups_for_summary(
    items: &[approval_claim::RegionalApprovalGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_freeze_summaries_from_group(items: &[approval_claim::RegionalApprovalGroup]) -> Vec<FreezeSummary> {
    let mut out: Vec<FreezeSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_summary(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_freeze_summary(item: &FreezeSummary) -> String {
    approval_claim::strict_approval_baseline_bucket(item.projected_volume())
}
