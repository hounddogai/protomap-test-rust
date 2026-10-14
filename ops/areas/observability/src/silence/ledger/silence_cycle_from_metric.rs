use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::metric as budget_burn_metric;

pub fn summarize_budget_burn_metrics_for_cycle(
    items: &[budget_burn_metric::BudgetBurnMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_silence_cycles_from_metric(items: &[budget_burn_metric::BudgetBurnMetric]) -> Vec<SilenceCycle> {
    let mut out: Vec<SilenceCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_cycle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_silence_cycle(item: &SilenceCycle) -> String {
    budget_burn_metric::normalize_budget_burn_entry_code(&item.code)
}
