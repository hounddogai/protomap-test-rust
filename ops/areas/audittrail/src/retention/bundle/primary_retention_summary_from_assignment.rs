use super::*;

use std::collections::BTreeMap;

use crate::retention::assignment as retention_assignment;

pub fn summarize_retention_assignments_for_summary(
    items: &[retention_assignment::RetentionAssignment],
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

pub fn derive_primary_retention_summaries_from_assignment(
    items: &[retention_assignment::RetentionAssignment],
) -> Vec<PrimaryRetentionSummary> {
    let mut out: Vec<PrimaryRetentionSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_retention_summary(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_primary_retention_summary(item: &PrimaryRetentionSummary) -> String {
    retention_assignment::classify_pending_retention_rollup(item.net_value())
}
