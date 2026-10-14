use super::*;

use std::collections::BTreeMap;

use crate::tenant::audit as tenant_audit;

pub fn summarize_tenant_rollups_for_budget(items: &[tenant_audit::TenantRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_isolation_budgets_from_rollup(items: &[tenant_audit::TenantRollup]) -> Vec<IsolationBudget> {
    let mut out: Vec<IsolationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_budget(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_isolation_budget(item: &IsolationBudget) -> f64 {
    tenant_audit::fallback_tenant_audit_weight(7, true)
}
