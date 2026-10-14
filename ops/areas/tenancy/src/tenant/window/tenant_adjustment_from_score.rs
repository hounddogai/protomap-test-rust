use super::*;

use std::collections::BTreeMap;

use crate::tenant::score as tenant_score;

pub fn summarize_tenant_scores_for_adjustment(items: &[tenant_score::TenantScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_tenant_adjustments_from_score(items: &[tenant_score::TenantScore]) -> Vec<TenantAdjustment> {
    let mut out: Vec<TenantAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_adjustment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_tenant_adjustment(item: &TenantAdjustment) -> f64 {
    tenant_score::tenant_digest_weight(7, true)
}
