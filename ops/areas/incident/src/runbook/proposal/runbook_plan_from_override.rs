use super::*;

use std::collections::BTreeMap;

use crate::page::override_item as page_override_item;

pub fn summarize_page_overrides_for_plan(items: &[page_override_item::PageOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_runbook_plans_from_override(items: &[page_override_item::PageOverride]) -> Vec<RunbookPlan> {
    let mut out: Vec<RunbookPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_plan(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_runbook_plan(item: &RunbookPlan) -> String {
    page_override_item::classify_capped_page_variance(item.measured_value())
}
