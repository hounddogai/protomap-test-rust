use super::*;

use std::collections::BTreeMap;

use crate::isolation::isolation_ledger as isolation_isolation_ledger;

pub fn summarize_global_isolation_claims_for_entry(
    items: &[isolation_isolation_ledger::GlobalIsolationClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_rolling_plan_entries_from_claim(
    items: &[isolation_isolation_ledger::GlobalIsolationClaim],
) -> Vec<RollingPlanEntry> {
    let mut out: Vec<RollingPlanEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_plan_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_rolling_plan_entry(item: &RollingPlanEntry) -> String {
    isolation_isolation_ledger::classify_scheduled_isolation_ledger(item.weighted_load())
}
