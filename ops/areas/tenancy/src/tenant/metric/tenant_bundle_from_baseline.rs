use super::*;

use std::collections::BTreeMap;

use crate::plan::audit as plan_audit;

pub fn summarize_plan_baselines_for_bundle(items: &[plan_audit::PlanBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_tenant_bundles_from_baseline(items: &[plan_audit::PlanBaseline]) -> Vec<TenantBundle> {
    let mut out: Vec<TenantBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_bundle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_tenant_bundle_extra(item: &TenantBundle) -> f64 {
    plan_audit::plan_milestone_weight(9, true)
}
