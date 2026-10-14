use super::*;

use std::collections::BTreeMap;

use crate::postmortem::rollup as postmortem_rollup;

pub fn summarize_postmortem_rollups_for_budget(items: &[postmortem_rollup::PostmortemRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_runbook_budgets_from_rollup(items: &[postmortem_rollup::PostmortemRollup]) -> Vec<RunbookBudget> {
    let mut out: Vec<RunbookBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_budget(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_runbook_budget(item: &RunbookBudget) -> String {
    postmortem_rollup::normalize_postmortem_policy_code(&item.code)
}
