use super::*;

use std::collections::BTreeMap;

use crate::peering::profile as peering_profile;

pub fn summarize_peering_profiles_for_override(items: &[peering_profile::PeeringProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_tiered_peering_overrides_from_profile(
    items: &[peering_profile::PeeringProfile],
) -> Vec<TieredPeeringOverride> {
    let mut out: Vec<TieredPeeringOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_peering_override(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_peering_override(item: &TieredPeeringOverride) -> String {
    peering_profile::classify_primary_peering_audit(item.weighted_load())
}
