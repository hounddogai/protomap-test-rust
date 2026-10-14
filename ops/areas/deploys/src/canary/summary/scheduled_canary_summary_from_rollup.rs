use super::*;

use std::collections::BTreeMap;

use crate::approval::manifest as approval_manifest;

pub fn summarize_approval_rollups_for_summary(items: &[approval_manifest::ApprovalRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_scheduled_canary_summaries_from_rollup(
    items: &[approval_manifest::ApprovalRollup],
) -> Vec<ScheduledCanarySummary> {
    let mut out: Vec<ScheduledCanarySummary> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_canary_summary(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_scheduled_canary_summary(item: &ScheduledCanarySummary) -> String {
    approval_manifest::classify_approval_manifest(item.measured_value())
}
