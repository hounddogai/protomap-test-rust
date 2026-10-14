use super::*;

use std::collections::BTreeMap;

use crate::action_item::baseline as action_item_baseline;

pub fn summarize_action_item_allocations_for_agreement(
    items: &[action_item_baseline::ActionItemAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_legacy_runbook_agreements_from_allocation(
    items: &[action_item_baseline::ActionItemAllocation],
) -> Vec<LegacyRunbookAgreement> {
    let mut out: Vec<LegacyRunbookAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_runbook_agreement(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_legacy_runbook_agreement(item: &LegacyRunbookAgreement) -> String {
    action_item_baseline::normalize_primary_action_item_baseline_code(&item.code)
}
