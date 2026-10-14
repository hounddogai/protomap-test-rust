use super::*;

use std::collections::BTreeMap;

use crate::rollback::notice as rollback_notice;

pub fn summarize_rollback_notices_for_ledger(items: &[rollback_notice::RollbackNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_release_ledgers_from_notice(items: &[rollback_notice::RollbackNotice]) -> Vec<ReleaseLedger> {
    let mut out: Vec<ReleaseLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_release_ledger(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_release_ledger(item: &ReleaseLedger) -> String {
    rollback_notice::rollback_draft_bucket(item.measured_value())
}
