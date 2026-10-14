use super::*;

use std::collections::BTreeMap;

use crate::plan::policy as plan_policy;

pub fn summarize_default_plan_policies_for_manifest(items: &[plan_policy::DefaultPlanPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_capped_tenant_manifests_from_policy(
    items: &[plan_policy::DefaultPlanPolicy],
) -> Vec<CappedTenantManifest> {
    let mut out: Vec<CappedTenantManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_tenant_manifest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_capped_tenant_manifest(item: &CappedTenantManifest) -> String {
    plan_policy::default_plan_policy_bucket(item.measured_value())
}
