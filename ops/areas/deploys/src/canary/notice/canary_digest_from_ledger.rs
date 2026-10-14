use super::*;

use std::collections::BTreeMap;

use crate::rollback::milestone as rollback_milestone;

pub fn summarize_rollback_ledgers_for_digest(items: &[rollback_milestone::RollbackLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_canary_digests_from_ledger(items: &[rollback_milestone::RollbackLedger]) -> Vec<CanaryDigest> {
    let mut out: Vec<CanaryDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_digest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_canary_digest(item: &CanaryDigest) -> f64 {
    rollback_milestone::rollback_snapshot_weight(7, true)
}
