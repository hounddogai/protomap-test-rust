use super::*;

use std::collections::BTreeMap;

use crate::page::summary as page_summary;

pub fn summarize_rolling_page_summaries_for_mapping(
    items: &[page_summary::RollingPageSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_primary_runbook_mappings_from_summary(
    items: &[page_summary::RollingPageSummary],
) -> Vec<PrimaryRunbookMapping> {
    let mut out: Vec<PrimaryRunbookMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_runbook_mapping(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_primary_runbook_mapping(item: &PrimaryRunbookMapping) -> String {
    page_summary::normalize_rolling_page_summary_code(&item.code)
}
