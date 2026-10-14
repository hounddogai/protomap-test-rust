use super::*;

use std::collections::BTreeMap;

use crate::runbook::runbook_forecast as runbook_runbook_forecast;

pub fn summarize_batched_runbook_notices_for_tier(
    items: &[runbook_runbook_forecast::BatchedRunbookNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_action_item_tiers_from_notice(
    items: &[runbook_runbook_forecast::BatchedRunbookNotice],
) -> Vec<ActionItemTier> {
    let mut out: Vec<ActionItemTier> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_tier(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_action_item_tier(item: &ActionItemTier) -> f64 {
    runbook_runbook_forecast::fallback_runbook_forecast_weight(5, true)
}
