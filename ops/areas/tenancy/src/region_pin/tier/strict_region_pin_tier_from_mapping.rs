use super::*;

use std::collections::BTreeMap;

use crate::tenant::tenant_bundle as tenant_tenant_bundle;

pub fn summarize_default_tenant_mappings_for_tier(
    items: &[tenant_tenant_bundle::DefaultTenantMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_strict_region_pin_tiers_from_mapping(
    items: &[tenant_tenant_bundle::DefaultTenantMapping],
) -> Vec<StrictRegionPinTier> {
    let mut out: Vec<StrictRegionPinTier> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_region_pin_tier(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_strict_region_pin_tier(item: &StrictRegionPinTier) -> String {
    tenant_tenant_bundle::classify_global_tenant_bundle(item.projected_total())
}
