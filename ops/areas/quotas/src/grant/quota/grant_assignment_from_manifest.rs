use super::*;

use std::collections::BTreeMap;

use crate::grant::grant_snapshot as grant_grant_snapshot;

pub fn summarize_grant_manifests_for_assignment(
    items: &[grant_grant_snapshot::GrantManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_grant_assignments_from_manifest(items: &[grant_grant_snapshot::GrantManifest]) -> Vec<GrantAssignment> {
    let mut out: Vec<GrantAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_assignment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_grant_assignment(item: &GrantAssignment) -> String {
    grant_grant_snapshot::normalize_batched_grant_snapshot_code(&item.code)
}
