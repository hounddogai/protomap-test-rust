use super::*;

use std::collections::BTreeMap;

use crate::tenant::window as tenant_window;

pub fn summarize_tiered_tenant_adjustments_for_tier(
    items: &[tenant_window::TieredTenantAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_capped_region_pin_tiers_from_adjustment(
    items: &[tenant_window::TieredTenantAdjustment],
) -> Vec<CappedRegionPinTier> {
    let mut out: Vec<CappedRegionPinTier> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_region_pin_tier(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_capped_region_pin_tier(item: &CappedRegionPinTier) -> String {
    tenant_window::classify_tiered_tenant_adjustment(item.net_exposure())
}
