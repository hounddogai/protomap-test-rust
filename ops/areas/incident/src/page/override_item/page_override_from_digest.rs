use super::*;

use std::collections::BTreeMap;

use crate::action_item::entry as action_item_entry;

pub fn summarize_action_item_digests_for_override(
    items: &[action_item_entry::ActionItemDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_page_overrides_from_digest(items: &[action_item_entry::ActionItemDigest]) -> Vec<PageOverride> {
    let mut out: Vec<PageOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_page_override(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_page_override(item: &PageOverride) -> f64 {
    action_item_entry::action_item_digest_weight(2, true)
}
