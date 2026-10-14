use super::*;

use std::collections::BTreeMap;

use crate::action_item::item_entry as action_item_item_entry;
use crate::runbook::review as runbook_review;

pub fn summarize_primary_runbook_entries_for_allocation(
    items: &[runbook_review::PrimaryRunbookEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_runbook_allocations_from_entry(items: &[runbook_review::PrimaryRunbookEntry]) -> Vec<RunbookAllocation> {
    let mut out: Vec<RunbookAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_allocation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_runbook_allocation(item: &RunbookAllocation) -> String {
    runbook_review::normalize_primary_runbook_entry_code(&item.code)
}

pub fn summarize_pending_action_item_entries_for_allocation(
    items: &[action_item_item_entry::PendingActionItemEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_runbook_allocations_from_entry_extra(
    items: &[action_item_item_entry::PendingActionItemEntry],
) -> Vec<RunbookAllocation> {
    let mut out: Vec<RunbookAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_allocation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_runbook_allocation(item: &RunbookAllocation) -> String {
    action_item_item_entry::pending_action_item_entry_bucket(item.net_value())
}
