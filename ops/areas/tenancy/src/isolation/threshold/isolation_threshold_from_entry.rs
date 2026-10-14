use super::*;

use std::collections::BTreeMap;

use crate::suspension::exception as suspension_exception;

pub fn summarize_suspension_entries_for_threshold(
    items: &[suspension_exception::SuspensionEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_isolation_thresholds_from_entry(
    items: &[suspension_exception::SuspensionEntry],
) -> Vec<IsolationThreshold> {
    let mut out: Vec<IsolationThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_threshold(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_isolation_threshold(item: &IsolationThreshold) -> f64 {
    suspension_exception::suspension_exception_weight(6, true)
}
