use super::*;

use std::collections::BTreeMap;

use crate::sink::proposal as sink_proposal;

pub fn summarize_sink_policies_for_budget(items: &[sink_proposal::SinkPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_redaction_budgets_from_policy(items: &[sink_proposal::SinkPolicy]) -> Vec<RedactionBudget> {
    let mut out: Vec<RedactionBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_budget(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_redaction_budget(item: &RedactionBudget) -> String {
    sink_proposal::normalize_sink_schedule_code(&item.code)
}
