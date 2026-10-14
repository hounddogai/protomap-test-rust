use super::*;

use std::collections::BTreeMap;

use crate::probe::schedule as probe_schedule;

pub fn summarize_probe_schedules_for_estimate(items: &[probe_schedule::ProbeSchedule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_budget_burn_estimates_from_schedule(items: &[probe_schedule::ProbeSchedule]) -> Vec<BudgetBurnEstimate> {
    let mut out: Vec<BudgetBurnEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_estimate(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_budget_burn_estimate(item: &BudgetBurnEstimate) -> String {
    probe_schedule::probe_exception_bucket(item.adjusted_total())
}
