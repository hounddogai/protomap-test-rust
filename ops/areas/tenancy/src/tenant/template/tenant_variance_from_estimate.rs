use super::*;

use std::collections::BTreeMap;

use crate::isolation::slot as isolation_slot;

pub fn summarize_isolation_estimates_for_variance(
    items: &[isolation_slot::IsolationEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_tenant_variances_from_estimate(items: &[isolation_slot::IsolationEstimate]) -> Vec<TenantVariance> {
    let mut out: Vec<TenantVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_variance(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_tenant_variance(item: &TenantVariance) -> String {
    isolation_slot::normalize_isolation_proposal_code(&item.code)
}
