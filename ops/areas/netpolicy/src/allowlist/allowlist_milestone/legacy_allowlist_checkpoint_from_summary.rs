use super::*;

use std::collections::BTreeMap;

use crate::peering::peering_summary as peering_peering_summary;

pub fn summarize_tiered_peering_summaries_for_checkpoint(
    items: &[peering_peering_summary::TieredPeeringSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_legacy_allowlist_checkpoints_from_summary(
    items: &[peering_peering_summary::TieredPeeringSummary],
) -> Vec<LegacyAllowlistCheckpoint> {
    let mut out: Vec<LegacyAllowlistCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_allowlist_checkpoint(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_legacy_allowlist_checkpoint(item: &LegacyAllowlistCheckpoint) -> String {
    peering_peering_summary::classify_peering_mapping(item.weighted_load())
}
