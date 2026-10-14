use super::*;

use std::collections::BTreeMap;

use crate::page::summary as page_summary;

pub fn summarize_rolling_page_summaries_for_plan(items: &[page_summary::RollingPageSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_strict_postmortem_plans_from_summary(
    items: &[page_summary::RollingPageSummary],
) -> Vec<StrictPostmortemPlan> {
    let mut out: Vec<StrictPostmortemPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_postmortem_plan(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_strict_postmortem_plan(item: &StrictPostmortemPlan) -> f64 {
    page_summary::lenient_page_group_weight(1, true)
}
