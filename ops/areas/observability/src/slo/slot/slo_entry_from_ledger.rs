use super::*;

use std::collections::BTreeMap;

use crate::silence::ledger as silence_ledger;

pub fn summarize_rolling_silence_ledgers_for_entry(
    items: &[silence_ledger::RollingSilenceLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_slo_entries_from_ledger(items: &[silence_ledger::RollingSilenceLedger]) -> Vec<SloEntry> {
    let mut out: Vec<SloEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_entry(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_slo_entry(item: &SloEntry) -> String {
    silence_ledger::classify_rolling_silence_ledger(item.measured_value())
}
