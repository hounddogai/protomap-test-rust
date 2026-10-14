use super::*;

use std::collections::BTreeMap;

use crate::peering::budget as peering_budget;

pub fn summarize_lenient_peering_variances_for_exception(
    items: &[peering_budget::LenientPeeringVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_tiered_peering_exceptions_from_variance(
    items: &[peering_budget::LenientPeeringVariance],
) -> Vec<TieredPeeringException> {
    let mut out: Vec<TieredPeeringException> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_peering_exception(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_tiered_peering_exception(item: &TieredPeeringException) -> String {
    peering_budget::classify_peering_budget(item.adjusted_load())
}
