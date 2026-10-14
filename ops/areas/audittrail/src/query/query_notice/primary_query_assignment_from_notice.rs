use super::*;

use std::collections::BTreeMap;

use crate::retention::retention_override as retention_retention_override;

pub fn summarize_regional_retention_notices_for_assignment(
    items: &[retention_retention_override::RegionalRetentionNotice],
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

pub fn derive_primary_query_assignments_from_notice(
    items: &[retention_retention_override::RegionalRetentionNotice],
) -> Vec<PrimaryQueryAssignment> {
    let mut out: Vec<PrimaryQueryAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_query_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_primary_query_assignment(item: &PrimaryQueryAssignment) -> String {
    retention_retention_override::classify_capped_retention_reservation(item.adjusted_total())
}
