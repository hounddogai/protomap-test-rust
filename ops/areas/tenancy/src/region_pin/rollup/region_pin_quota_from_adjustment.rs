use super::*;

use std::collections::BTreeMap;

use crate::tenant::rollup as tenant_rollup;

pub fn summarize_strict_tenant_adjustments_for_quota(
    items: &[tenant_rollup::StrictTenantAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_region_pin_quotas_from_adjustment(
    items: &[tenant_rollup::StrictTenantAdjustment],
) -> Vec<RegionPinQuota> {
    let mut out: Vec<RegionPinQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_region_pin_quota(item: &RegionPinQuota) -> String {
    tenant_rollup::normalize_rolling_tenant_rollup_code(&item.code)
}
