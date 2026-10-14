use super::*;

use std::collections::BTreeMap;

use crate::plan::template as plan_template;

pub fn summarize_tiered_plan_drafts_for_bundle(items: &[plan_template::TieredPlanDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_batched_tenant_bundles_from_draft(items: &[plan_template::TieredPlanDraft]) -> Vec<BatchedTenantBundle> {
    let mut out: Vec<BatchedTenantBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_tenant_bundle(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_batched_tenant_bundle(item: &BatchedTenantBundle) -> f64 {
    plan_template::lenient_plan_quota_weight(1, true)
}
