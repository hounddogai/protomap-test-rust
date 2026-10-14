use super::*;

use std::collections::BTreeMap;

use crate::canary::notice as canary_notice;

pub fn summarize_canary_mappings_for_ledger(items: &[canary_notice::CanaryMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_release_ledgers_from_mapping(items: &[canary_notice::CanaryMapping]) -> Vec<ReleaseLedger> {
    let mut out: Vec<ReleaseLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_release_ledger(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_release_ledger(item: &ReleaseLedger) -> String {
    canary_notice::canary_allocation_bucket(item.measured_value())
}
