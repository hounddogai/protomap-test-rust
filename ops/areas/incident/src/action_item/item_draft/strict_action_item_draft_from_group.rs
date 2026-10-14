use super::*;

use std::collections::BTreeMap;

use crate::escalation::slot as escalation_slot;

pub fn summarize_primary_escalation_groups_for_draft(
    items: &[escalation_slot::PrimaryEscalationGroup],
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

pub fn derive_strict_action_item_drafts_from_group(
    items: &[escalation_slot::PrimaryEscalationGroup],
) -> Vec<StrictActionItemDraft> {
    let mut out: Vec<StrictActionItemDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_action_item_draft(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_strict_action_item_draft(item: &StrictActionItemDraft) -> String {
    escalation_slot::classify_escalation_slot(item.measured_value())
}
