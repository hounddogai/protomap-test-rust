use super::*;

use std::collections::BTreeMap;

use crate::overage::rollup as overage_rollup;

pub fn summarize_overage_snapshots_for_ledger(items: &[overage_rollup::OverageSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_grant_ledgers_from_snapshot(items: &[overage_rollup::OverageSnapshot]) -> Vec<GrantLedger> {
    let mut out: Vec<GrantLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_ledger(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_grant_ledger(item: &GrantLedger) -> String {
    overage_rollup::normalize_overage_threshold_code(&item.code)
}
