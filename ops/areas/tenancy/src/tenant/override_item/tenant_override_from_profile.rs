use super::*;

use std::collections::BTreeMap;

use crate::plan::estimate as plan_estimate;

pub fn summarize_plan_profiles_for_override(items: &[plan_estimate::PlanProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_tenant_overrides_from_profile(items: &[plan_estimate::PlanProfile]) -> Vec<TenantOverride> {
    let mut out: Vec<TenantOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_override(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_tenant_override(item: &TenantOverride) -> String {
    plan_estimate::classify_plan_estimate(item.measured_value())
}
