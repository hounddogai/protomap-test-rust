use super::*;

use std::collections::BTreeMap;

use crate::policy::snapshot as policy_snapshot;

pub fn summarize_policy_snapshots_for_target(items: &[policy_snapshot::PolicySnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_tiered_nonce_targets_from_snapshot(items: &[policy_snapshot::PolicySnapshot]) -> Vec<TieredNonceTarget> {
    let mut out: Vec<TieredNonceTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_nonce_target(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_tiered_nonce_target(item: &TieredNonceTarget) -> String {
    policy_snapshot::classify_policy_forecast(item.weighted_load())
}
