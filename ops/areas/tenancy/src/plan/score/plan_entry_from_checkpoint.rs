use super::*;

use std::collections::BTreeMap;

use crate::tenant::ledger as tenant_ledger;

pub fn summarize_tenant_checkpoints_for_entry(items: &[tenant_ledger::TenantCheckpoint]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_plan_entries_from_checkpoint(items: &[tenant_ledger::TenantCheckpoint]) -> Vec<PlanEntry> {
    let mut out: Vec<PlanEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_entry(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_plan_entry(item: &PlanEntry) -> f64 {
    tenant_ledger::tenant_allocation_weight(5, true)
}
