use super::*;

use std::collections::BTreeMap;

use crate::query::preference as query_preference;

pub fn summarize_capped_query_rollups_for_plan(items: &[query_preference::CappedQueryRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_retention_plans_from_rollup(items: &[query_preference::CappedQueryRollup]) -> Vec<RetentionPlan> {
    let mut out: Vec<RetentionPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_plan(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_retention_plan(item: &RetentionPlan) -> f64 {
    query_preference::query_summary_weight(2, true)
}
