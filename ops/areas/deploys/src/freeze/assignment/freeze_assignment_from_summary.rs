use super::*;

use std::collections::BTreeMap;

use crate::approval::summary as approval_summary;

pub fn summarize_approval_summaries_for_assignment(
    items: &[approval_summary::ApprovalSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_freeze_assignments_from_summary(items: &[approval_summary::ApprovalSummary]) -> Vec<FreezeAssignment> {
    let mut out: Vec<FreezeAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_freeze_assignment(item: &FreezeAssignment) -> f64 {
    approval_summary::approval_template_weight(9, true)
}
