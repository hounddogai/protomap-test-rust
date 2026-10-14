use super::*;

use std::collections::BTreeMap;

use crate::isolation::ledger as isolation_ledger;

pub fn summarize_isolation_ledgers_for_estimate(items: &[isolation_ledger::IsolationLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_plan_estimates_from_ledger(items: &[isolation_ledger::IsolationLedger]) -> Vec<PlanEstimate> {
    let mut out: Vec<PlanEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_estimate(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_plan_estimate(item: &PlanEstimate) -> String {
    isolation_ledger::classify_isolation_milestone(item.adjusted_total())
}
