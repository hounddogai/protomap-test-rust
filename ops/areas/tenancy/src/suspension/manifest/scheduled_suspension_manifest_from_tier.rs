use super::*;

use std::collections::BTreeMap;

use crate::tenant::exception as tenant_exception;

pub fn summarize_tenant_tiers_for_manifest(items: &[tenant_exception::TenantTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_scheduled_suspension_manifests_from_tier(
    items: &[tenant_exception::TenantTier],
) -> Vec<ScheduledSuspensionManifest> {
    let mut out: Vec<ScheduledSuspensionManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_suspension_manifest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_scheduled_suspension_manifest(item: &ScheduledSuspensionManifest) -> f64 {
    tenant_exception::tenant_tier_weight(3, true)
}
