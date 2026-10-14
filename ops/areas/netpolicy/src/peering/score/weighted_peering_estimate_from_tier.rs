use super::*;

use std::collections::BTreeMap;

use crate::peering::record_item as peering_record_item;

pub fn summarize_global_peering_tiers_for_estimate(
    items: &[peering_record_item::GlobalPeeringTier],
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

pub fn derive_weighted_peering_estimates_from_tier(
    items: &[peering_record_item::GlobalPeeringTier],
) -> Vec<WeightedPeeringEstimate> {
    let mut out: Vec<WeightedPeeringEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_peering_estimate(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_weighted_peering_estimate(item: &WeightedPeeringEstimate) -> f64 {
    peering_record_item::capped_peering_record_weight(5, true)
}
