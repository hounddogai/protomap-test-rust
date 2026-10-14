use super::*;

use std::collections::BTreeMap;

use crate::tenant::mapping as tenant_mapping;

pub fn summarize_tenant_mappings_for_profile(items: &[tenant_mapping::TenantMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_scheduled_region_pin_profiles_from_mapping(
    items: &[tenant_mapping::TenantMapping],
) -> Vec<ScheduledRegionPinProfile> {
    let mut out: Vec<ScheduledRegionPinProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_region_pin_profile(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_scheduled_region_pin_profile(item: &ScheduledRegionPinProfile) -> String {
    tenant_mapping::normalize_tenant_rule_code(&item.code)
}
