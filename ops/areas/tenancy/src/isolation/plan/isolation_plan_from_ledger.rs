use super::*;

use std::collections::BTreeMap;

use crate::tenant::ledger as tenant_ledger;

pub fn summarize_tenant_ledgers_for_plan(items: &[tenant_ledger::TenantLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_isolation_plans_from_ledger(items: &[tenant_ledger::TenantLedger]) -> Vec<IsolationPlan> {
    let mut out: Vec<IsolationPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_plan(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_isolation_plan(item: &IsolationPlan) -> String {
    tenant_ledger::classify_tenant_checkpoint(item.measured_value())
}
