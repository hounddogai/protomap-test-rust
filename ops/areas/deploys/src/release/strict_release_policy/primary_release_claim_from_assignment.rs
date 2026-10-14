use super::*;

use std::collections::BTreeMap;

use crate::rollback::assignment as rollback_assignment;

pub fn summarize_regional_rollback_assignments_for_claim(
    items: &[rollback_assignment::RegionalRollbackAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_primary_release_claims_from_assignment(
    items: &[rollback_assignment::RegionalRollbackAssignment],
) -> Vec<PrimaryReleaseClaim> {
    let mut out: Vec<PrimaryReleaseClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_release_claim(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_primary_release_claim(item: &PrimaryReleaseClaim) -> String {
    rollback_assignment::weighted_rollback_bundle_bucket(item.projected_load())
}
