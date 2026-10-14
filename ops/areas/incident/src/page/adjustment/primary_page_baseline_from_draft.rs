use super::*;

use std::collections::BTreeMap;

use crate::action_item::item_draft as action_item_item_draft;

pub fn summarize_strict_action_item_drafts_for_baseline(
    items: &[action_item_item_draft::StrictActionItemDraft],
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

pub fn derive_primary_page_baselines_from_draft(
    items: &[action_item_item_draft::StrictActionItemDraft],
) -> Vec<PrimaryPageBaseline> {
    let mut out: Vec<PrimaryPageBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_page_baseline(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_primary_page_baseline(item: &PrimaryPageBaseline) -> String {
    action_item_item_draft::classify_scheduled_action_item_adjustment(item.weighted_value())
}
