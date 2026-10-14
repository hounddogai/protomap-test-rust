use super::*;

use std::collections::BTreeMap;

use crate::isolation::rollup as isolation_rollup;

pub fn summarize_isolation_assignments_for_digest(
    items: &[isolation_rollup::IsolationAssignment],
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

pub fn derive_global_suspension_digests_from_assignment(
    items: &[isolation_rollup::IsolationAssignment],
) -> Vec<GlobalSuspensionDigest> {
    let mut out: Vec<GlobalSuspensionDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_global_suspension_digest(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_global_suspension_digest(item: &GlobalSuspensionDigest) -> String {
    isolation_rollup::classify_lenient_isolation_rollup(item.projected_exposure())
}
