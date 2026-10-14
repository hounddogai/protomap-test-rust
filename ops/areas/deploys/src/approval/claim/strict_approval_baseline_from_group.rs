use super::*;

use std::collections::BTreeMap;

use crate::approval::preference as approval_preference;

pub fn summarize_approval_groups_for_baseline(items: &[approval_preference::ApprovalGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_strict_approval_baselines_from_group(
    items: &[approval_preference::ApprovalGroup],
) -> Vec<StrictApprovalBaseline> {
    let mut out: Vec<StrictApprovalBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_approval_baseline(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_strict_approval_baseline(item: &StrictApprovalBaseline) -> String {
    approval_preference::classify_approval_group(item.measured_value())
}
