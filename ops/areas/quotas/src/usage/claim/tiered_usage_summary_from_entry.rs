use super::*;

use std::collections::BTreeMap;

use crate::usage::entry as usage_entry;

pub fn summarize_usage_entries_for_summary(items: &[usage_entry::UsageEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_tiered_usage_summaries_from_entry(items: &[usage_entry::UsageEntry]) -> Vec<TieredUsageSummary> {
    let mut out: Vec<TieredUsageSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_usage_summary(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_tiered_usage_summary(item: &TieredUsageSummary) -> String {
    usage_entry::normalize_usage_entry_code(&item.code)
}
