use super::*;

use std::collections::BTreeMap;

use crate::tenant::preference as tenant_preference;

pub fn summarize_adaptive_tenant_rollups_for_tier(
    items: &[tenant_preference::AdaptiveTenantRollup],
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

pub fn derive_batched_suspension_tiers_from_rollup(
    items: &[tenant_preference::AdaptiveTenantRollup],
) -> Vec<BatchedSuspensionTier> {
    let mut out: Vec<BatchedSuspensionTier> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_suspension_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_batched_suspension_tier(item: &BatchedSuspensionTier) -> f64 {
    tenant_preference::adaptive_tenant_rollup_weight(4, true)
}
