use super::*;

use std::collections::BTreeMap;

use crate::peering::snapshot as peering_snapshot;

pub fn summarize_peering_claims_for_rollup(items: &[peering_snapshot::PeeringClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_legacy_egress_rollups_from_claim(items: &[peering_snapshot::PeeringClaim]) -> Vec<LegacyEgressRollup> {
    let mut out: Vec<LegacyEgressRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_egress_rollup(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_legacy_egress_rollup(item: &LegacyEgressRollup) -> f64 {
    peering_snapshot::peering_claim_weight(2, true)
}
