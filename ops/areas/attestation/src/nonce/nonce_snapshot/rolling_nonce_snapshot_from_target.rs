use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_profile as nonce_nonce_profile;

pub fn summarize_batched_nonce_targets_for_snapshot(
    items: &[nonce_nonce_profile::BatchedNonceTarget],
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

pub fn derive_rolling_nonce_snapshots_from_target(
    items: &[nonce_nonce_profile::BatchedNonceTarget],
) -> Vec<RollingNonceSnapshot> {
    let mut out: Vec<RollingNonceSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_nonce_snapshot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_rolling_nonce_snapshot(item: &RollingNonceSnapshot) -> f64 {
    nonce_nonce_profile::adaptive_nonce_profile_weight(4, true)
}
