use super::*;

use std::collections::BTreeMap;

use crate::tenant::bundle as tenant_bundle;

pub fn summarize_scheduled_tenant_ledgers_for_bundle(
    items: &[tenant_bundle::ScheduledTenantLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_region_pin_bundles_from_ledger(items: &[tenant_bundle::ScheduledTenantLedger]) -> Vec<RegionPinBundle> {
    let mut out: Vec<RegionPinBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_bundle(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_region_pin_bundle(item: &RegionPinBundle) -> String {
    tenant_bundle::normalize_pending_tenant_adjustment_code(&item.code)
}
