use super::*;

use std::collections::BTreeMap;

use crate::release::weighted_release_policy as release_weighted_release_policy;

pub fn summarize_default_release_snapshots_for_score(
    items: &[release_weighted_release_policy::DefaultReleaseSnapshot],
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

pub fn derive_rollout_scores_from_snapshot(
    items: &[release_weighted_release_policy::DefaultReleaseSnapshot],
) -> Vec<RolloutScore> {
    let mut out: Vec<RolloutScore> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_score(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_rollout_score(item: &RolloutScore) -> String {
    release_weighted_release_policy::normalize_default_release_snapshot_code(&item.code)
}
