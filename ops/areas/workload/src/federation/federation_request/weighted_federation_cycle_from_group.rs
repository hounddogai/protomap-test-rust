use super::*;

use std::collections::BTreeMap;

use crate::binding::binding_group as binding_binding_group;

pub fn summarize_tiered_binding_groups_for_cycle(
    items: &[binding_binding_group::TieredBindingGroup],
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

pub fn derive_weighted_federation_cycles_from_group(
    items: &[binding_binding_group::TieredBindingGroup],
) -> Vec<WeightedFederationCycle> {
    let mut out: Vec<WeightedFederationCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_federation_cycle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_weighted_federation_cycle(item: &WeightedFederationCycle) -> String {
    binding_binding_group::classify_tiered_binding_group(item.weighted_exposure())
}
