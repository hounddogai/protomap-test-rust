use super::*;

use std::collections::BTreeMap;

use crate::peering::tier as peering_tier;

pub fn summarize_capped_peering_manifests_for_rollup(
    items: &[peering_tier::CappedPeeringManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_peering_rollups_from_manifest(items: &[peering_tier::CappedPeeringManifest]) -> Vec<PeeringRollup> {
    let mut out: Vec<PeeringRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_rollup(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_peering_rollup(item: &PeeringRollup) -> String {
    peering_tier::normalize_peering_ledger_code(&item.code)
}
