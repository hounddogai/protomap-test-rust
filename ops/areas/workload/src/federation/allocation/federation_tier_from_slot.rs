use super::*;

use std::collections::BTreeMap;

use crate::binding::audit as binding_audit;

pub fn summarize_legacy_binding_slots_for_tier(items: &[binding_audit::LegacyBindingSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_federation_tiers_from_slot(items: &[binding_audit::LegacyBindingSlot]) -> Vec<FederationTier> {
    let mut out: Vec<FederationTier> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_tier(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_federation_tier(item: &FederationTier) -> String {
    binding_audit::legacy_binding_slot_bucket(item.projected_load())
}
