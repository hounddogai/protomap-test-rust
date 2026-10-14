use super::*;

use std::collections::BTreeMap;

use crate::postmortem::draft as postmortem_draft;

pub fn summarize_fallback_postmortem_entries_for_summary(
    items: &[postmortem_draft::FallbackPostmortemEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_rolling_action_item_summaries_from_entry(
    items: &[postmortem_draft::FallbackPostmortemEntry],
) -> Vec<RollingActionItemSummary> {
    let mut out: Vec<RollingActionItemSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_action_item_summary(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_rolling_action_item_summary(item: &RollingActionItemSummary) -> String {
    postmortem_draft::normalize_lenient_postmortem_draft_code(&item.code)
}
