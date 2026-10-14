use super::*;

use std::collections::BTreeMap;

use crate::isolation::summary as isolation_summary;

pub fn summarize_manual_isolation_thresholds_for_exception(
    items: &[isolation_summary::ManualIsolationThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_rolling_suspension_exceptions_from_threshold(
    items: &[isolation_summary::ManualIsolationThreshold],
) -> Vec<RollingSuspensionException> {
    let mut out: Vec<RollingSuspensionException> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_suspension_exception(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_rolling_suspension_exception(item: &RollingSuspensionException) -> String {
    isolation_summary::legacy_isolation_milestone_bucket(item.effective_total())
}
