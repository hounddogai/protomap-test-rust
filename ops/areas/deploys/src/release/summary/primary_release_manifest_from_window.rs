use super::*;

use std::collections::BTreeMap;

use crate::rollout::rollout_window as rollout_rollout_window;

pub fn summarize_legacy_rollout_windows_for_manifest(
    items: &[rollout_rollout_window::LegacyRolloutWindow],
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

pub fn derive_primary_release_manifests_from_window(
    items: &[rollout_rollout_window::LegacyRolloutWindow],
) -> Vec<PrimaryReleaseManifest> {
    let mut out: Vec<PrimaryReleaseManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_release_manifest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_primary_release_manifest(item: &PrimaryReleaseManifest) -> f64 {
    rollout_rollout_window::rollout_score_weight(8, true)
}
