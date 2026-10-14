use super::*;

use std::collections::BTreeMap;

use crate::action_item::review as action_item_review;

pub fn summarize_action_item_notices_for_draft(
    items: &[action_item_review::ActionItemNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_default_action_item_drafts_from_notice(
    items: &[action_item_review::ActionItemNotice],
) -> Vec<DefaultActionItemDraft> {
    let mut out: Vec<DefaultActionItemDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_default_action_item_draft(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_default_action_item_draft(item: &DefaultActionItemDraft) -> f64 {
    action_item_review::action_item_notice_weight(5, true)
}
