use super::*;

use std::collections::BTreeMap;

use crate::tenant::tenant_claim as tenant_tenant_claim;

pub fn summarize_tenant_quotas_for_slot(items: &[tenant_tenant_claim::TenantQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_isolation_slots_from_quota(items: &[tenant_tenant_claim::TenantQuota]) -> Vec<IsolationSlot> {
    let mut out: Vec<IsolationSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_slot(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_isolation_slot(item: &IsolationSlot) -> String {
    tenant_tenant_claim::tenant_schedule_bucket(item.net_volume())
}
