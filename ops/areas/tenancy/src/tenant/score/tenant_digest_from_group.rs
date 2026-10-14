use super::*;

use std::collections::BTreeMap;

use crate::isolation::plan as isolation_plan;

pub fn summarize_isolation_groups_for_digest(items: &[isolation_plan::IsolationGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_tenant_digests_from_group(items: &[isolation_plan::IsolationGroup]) -> Vec<TenantDigest> {
    let mut out: Vec<TenantDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_digest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_tenant_digest(item: &TenantDigest) -> f64 {
    isolation_plan::isolation_plan_weight(9, true)
}
