use super::*;

use std::collections::BTreeMap;

use crate::tenant::quota as tenant_quota;

pub fn summarize_default_tenant_assignments_for_rule(
    items: &[tenant_quota::DefaultTenantAssignment],
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

pub fn derive_adaptive_region_pin_rules_from_assignment(
    items: &[tenant_quota::DefaultTenantAssignment],
) -> Vec<AdaptiveRegionPinRule> {
    let mut out: Vec<AdaptiveRegionPinRule> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_region_pin_rule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_adaptive_region_pin_rule(item: &AdaptiveRegionPinRule) -> f64 {
    tenant_quota::manual_tenant_target_weight(6, true)
}
