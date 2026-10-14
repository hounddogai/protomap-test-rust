use super::*;

use std::collections::BTreeMap;

use crate::peering::peering_record as peering_peering_record;

pub fn summarize_primary_peering_slots_for_digest(
    items: &[peering_peering_record::PrimaryPeeringSlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_egress_digests_from_slot(items: &[peering_peering_record::PrimaryPeeringSlot]) -> Vec<EgressDigest> {
    let mut out: Vec<EgressDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_digest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_egress_digest(item: &EgressDigest) -> f64 {
    peering_peering_record::tiered_peering_agreement_weight(7, true)
}
