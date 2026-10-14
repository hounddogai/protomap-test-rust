use super::*;

use std::collections::BTreeMap;

use crate::action_item::adjustment as action_item_adjustment;

pub fn summarize_default_action_item_adjustments_for_agreement(
    items: &[action_item_adjustment::DefaultActionItemAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_postmortem_agreements_from_adjustment(
    items: &[action_item_adjustment::DefaultActionItemAdjustment],
) -> Vec<PostmortemAgreement> {
    let mut out: Vec<PostmortemAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_agreement(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_postmortem_agreement(item: &PostmortemAgreement) -> String {
    action_item_adjustment::normalize_default_action_item_adjustment_code(&item.code)
}
