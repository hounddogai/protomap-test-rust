use super::*;

use std::collections::BTreeMap;

use crate::tenant::ledger as tenant_ledger;

pub fn summarize_tenant_allocations_for_quota(items: &[tenant_ledger::TenantAllocation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_suspension_quotas_from_allocation(items: &[tenant_ledger::TenantAllocation]) -> Vec<SuspensionQuota> {
    let mut out: Vec<SuspensionQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_quota(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_suspension_quota(item: &SuspensionQuota) -> f64 {
    tenant_ledger::tenant_audit_weight(4, true)
}
