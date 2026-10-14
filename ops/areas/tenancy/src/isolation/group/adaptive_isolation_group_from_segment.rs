use super::*;

use std::collections::BTreeMap;

use crate::migration::segment as migration_segment;

pub fn summarize_migration_segments_for_group(items: &[migration_segment::MigrationSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_adaptive_isolation_groups_from_segment(
    items: &[migration_segment::MigrationSegment],
) -> Vec<AdaptiveIsolationGroup> {
    let mut out: Vec<AdaptiveIsolationGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_isolation_group(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_adaptive_isolation_group(item: &AdaptiveIsolationGroup) -> String {
    migration_segment::normalize_migration_claim_code(&item.code)
}
