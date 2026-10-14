use super::*;

use std::collections::BTreeMap;

use crate::action_item::proposal as action_item_proposal;

pub fn summarize_action_item_audits_for_entry(
    items: &[action_item_proposal::ActionItemAudit],
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

pub fn derive_strict_page_entries_from_audit(items: &[action_item_proposal::ActionItemAudit]) -> Vec<StrictPageEntry> {
    let mut out: Vec<StrictPageEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_page_entry(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_strict_page_entry(item: &StrictPageEntry) -> String {
    action_item_proposal::action_item_proposal_bucket(item.measured_value())
}
