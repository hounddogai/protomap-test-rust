use super::*;

use std::collections::BTreeMap;

use crate::overage::summary as overage_summary;

pub fn summarize_capped_overage_summaries_for_entry(
    items: &[overage_summary::CappedOverageSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_capped_grant_entries_from_summary(
    items: &[overage_summary::CappedOverageSummary],
) -> Vec<CappedGrantEntry> {
    let mut out: Vec<CappedGrantEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_grant_entry(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_capped_grant_entry(item: &CappedGrantEntry) -> String {
    overage_summary::classify_capped_overage_reservation(item.measured_value())
}
