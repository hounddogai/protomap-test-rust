use super::*;

use std::collections::BTreeMap;

use crate::runbook::entry as runbook_entry;

pub fn summarize_tiered_runbook_templates_for_estimate(
    items: &[runbook_entry::TieredRunbookTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_action_item_estimates_from_template(
    items: &[runbook_entry::TieredRunbookTemplate],
) -> Vec<ActionItemEstimate> {
    let mut out: Vec<ActionItemEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_estimate(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_action_item_estimate(item: &ActionItemEstimate) -> String {
    runbook_entry::normalize_runbook_entry_code(&item.code)
}
