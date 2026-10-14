use super::*;

use std::collections::BTreeMap;

use crate::plan::slot as plan_slot;

pub fn summarize_plan_reviews_for_manifest(items: &[plan_slot::PlanReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_tenant_manifests_from_review(items: &[plan_slot::PlanReview]) -> Vec<TenantManifest> {
    let mut out: Vec<TenantManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_manifest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_tenant_manifest(item: &TenantManifest) -> String {
    plan_slot::normalize_plan_review_code(&item.code)
}
