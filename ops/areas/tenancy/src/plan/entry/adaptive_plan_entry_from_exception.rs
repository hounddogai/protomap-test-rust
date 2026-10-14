use super::*;

use std::collections::BTreeMap;

use crate::suspension::profile as suspension_profile;

pub fn summarize_rolling_suspension_exceptions_for_entry(
    items: &[suspension_profile::RollingSuspensionException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_adaptive_plan_entries_from_exception(
    items: &[suspension_profile::RollingSuspensionException],
) -> Vec<AdaptivePlanEntry> {
    let mut out: Vec<AdaptivePlanEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_plan_entry(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_adaptive_plan_entry(item: &AdaptivePlanEntry) -> String {
    suspension_profile::rolling_suspension_exception_bucket(item.adjusted_value())
}
