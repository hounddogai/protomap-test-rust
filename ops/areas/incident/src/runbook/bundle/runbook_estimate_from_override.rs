use super::*;

use std::collections::BTreeMap;

use crate::action_item::notice as action_item_notice;

pub fn summarize_fallback_action_item_overrides_for_estimate(
    items: &[action_item_notice::FallbackActionItemOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_runbook_estimates_from_override(
    items: &[action_item_notice::FallbackActionItemOverride],
) -> Vec<RunbookEstimate> {
    let mut out: Vec<RunbookEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_estimate(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_runbook_estimate(item: &RunbookEstimate) -> f64 {
    action_item_notice::fallback_action_item_override_weight(3, true)
}
