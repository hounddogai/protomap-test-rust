use super::*;

use std::collections::BTreeMap;

use crate::tenant::batched_tenant_audit as tenant_batched_tenant_audit;

pub fn summarize_tenant_profiles_for_group(
    items: &[tenant_batched_tenant_audit::TenantProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_suspension_groups_from_profile(
    items: &[tenant_batched_tenant_audit::TenantProfile],
) -> Vec<SuspensionGroup> {
    let mut out: Vec<SuspensionGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_group(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_suspension_group(item: &SuspensionGroup) -> String {
    tenant_batched_tenant_audit::classify_fallback_tenant_agreement(item.weighted_value())
}
