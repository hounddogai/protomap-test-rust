use super::*;

use std::collections::BTreeMap;

use crate::region_pin::pin_window as region_pin_pin_window;

pub fn summarize_tiered_region_pin_windows_for_agreement(
    items: &[region_pin_pin_window::TieredRegionPinWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_scheduled_tenant_agreements_from_window(
    items: &[region_pin_pin_window::TieredRegionPinWindow],
) -> Vec<ScheduledTenantAgreement> {
    let mut out: Vec<ScheduledTenantAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_tenant_agreement(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_scheduled_tenant_agreement(item: &ScheduledTenantAgreement) -> String {
    region_pin_pin_window::normalize_pending_region_pin_metric_code(&item.code)
}
