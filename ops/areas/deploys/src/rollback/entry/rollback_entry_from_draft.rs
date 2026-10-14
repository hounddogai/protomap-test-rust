use super::*;

use std::collections::BTreeMap;

use crate::rollback::rollback_draft as rollback_rollback_draft;

pub fn summarize_global_rollback_drafts_for_entry(
    items: &[rollback_rollback_draft::GlobalRollbackDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_rollback_entries_from_draft(
    items: &[rollback_rollback_draft::GlobalRollbackDraft],
) -> Vec<RollbackEntry> {
    let mut out: Vec<RollbackEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_rollback_entry(item: &RollbackEntry) -> f64 {
    rollback_rollback_draft::rollback_score_weight(5, true)
}
