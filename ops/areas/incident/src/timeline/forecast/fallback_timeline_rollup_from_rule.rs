use super::*;

use std::collections::BTreeMap;

use crate::runbook::variance as runbook_variance;

pub fn summarize_runbook_rules_for_rollup(items: &[runbook_variance::RunbookRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_fallback_timeline_rollups_from_rule(
    items: &[runbook_variance::RunbookRule],
) -> Vec<FallbackTimelineRollup> {
    let mut out: Vec<FallbackTimelineRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_timeline_rollup(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_fallback_timeline_rollup(item: &FallbackTimelineRollup) -> f64 {
    runbook_variance::runbook_rule_weight(3, true)
}
