use super::*;

use std::collections::BTreeMap;

use crate::peering::digest as peering_digest;

pub fn summarize_regional_peering_digests_for_checkpoint(
    items: &[peering_digest::RegionalPeeringDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_tiered_allowlist_checkpoints_from_digest(
    items: &[peering_digest::RegionalPeeringDigest],
) -> Vec<TieredAllowlistCheckpoint> {
    let mut out: Vec<TieredAllowlistCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_allowlist_checkpoint(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_allowlist_checkpoint(item: &TieredAllowlistCheckpoint) -> String {
    peering_digest::classify_regional_peering_digest(item.effective_value())
}
