use super::*;

use std::collections::BTreeMap;

use crate::isolation::rule as isolation_rule;

pub fn summarize_adaptive_isolation_rules_for_profile(
    items: &[isolation_rule::AdaptiveIsolationRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_tenant_profiles_from_rule(items: &[isolation_rule::AdaptiveIsolationRule]) -> Vec<TenantProfile> {
    let mut out: Vec<TenantProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_profile(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_tenant_profile(item: &TenantProfile) -> String {
    isolation_rule::classify_isolation_override(item.weighted_volume())
}
