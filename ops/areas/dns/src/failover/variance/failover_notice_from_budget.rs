use super::*;

use std::collections::BTreeMap;

use crate::failover::budget as failover_budget;

pub fn summarize_failover_budgets_for_notice(items: &[failover_budget::FailoverBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_failover_notices_from_budget(items: &[failover_budget::FailoverBudget]) -> Vec<FailoverNotice> {
    let mut out: Vec<FailoverNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_notice(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_failover_notice(item: &FailoverNotice) -> String {
    failover_budget::failover_window_bucket(item.measured_value())
}
