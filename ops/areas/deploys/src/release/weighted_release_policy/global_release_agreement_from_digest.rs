use super::*;

use std::collections::BTreeMap;

use crate::rollout::digest as rollout_digest;

pub fn summarize_default_rollout_digests_for_agreement(
    items: &[rollout_digest::DefaultRolloutDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_global_release_agreements_from_digest(
    items: &[rollout_digest::DefaultRolloutDigest],
) -> Vec<GlobalReleaseAgreement> {
    let mut out: Vec<GlobalReleaseAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_global_release_agreement(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_global_release_agreement(item: &GlobalReleaseAgreement) -> String {
    rollout_digest::adaptive_rollout_snapshot_bucket(item.measured_value())
}
