use super::*;

use std::collections::BTreeMap;

use crate::tenant::tenant_score as tenant_tenant_score;

pub fn summarize_primary_tenant_scores_for_window(
    items: &[tenant_tenant_score::PrimaryTenantScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_weighted_plan_windows_from_score(
    items: &[tenant_tenant_score::PrimaryTenantScore],
) -> Vec<WeightedPlanWindow> {
    let mut out: Vec<WeightedPlanWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_plan_window(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_weighted_plan_window(item: &WeightedPlanWindow) -> String {
    tenant_tenant_score::normalize_scheduled_tenant_agreement_code(&item.code)
}
