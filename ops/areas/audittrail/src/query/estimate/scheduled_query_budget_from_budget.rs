use super::*;

use std::collections::BTreeMap;

use crate::sink::plan as sink_plan;

pub fn summarize_sink_budgets_for_budget(items: &[sink_plan::SinkBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_scheduled_query_budgets_from_budget(items: &[sink_plan::SinkBudget]) -> Vec<ScheduledQueryBudget> {
    let mut out: Vec<ScheduledQueryBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_query_budget(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_scheduled_query_budget(item: &ScheduledQueryBudget) -> f64 {
    sink_plan::weighted_sink_digest_weight(7, true)
}
