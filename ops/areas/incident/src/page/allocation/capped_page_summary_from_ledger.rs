use super::*;

use std::collections::BTreeMap;

use crate::page::variance as page_variance;

pub fn summarize_legacy_page_ledgers_for_summary(items: &[page_variance::LegacyPageLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_capped_page_summaries_from_ledger(items: &[page_variance::LegacyPageLedger]) -> Vec<CappedPageSummary> {
    let mut out: Vec<CappedPageSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_page_summary(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_capped_page_summary(item: &CappedPageSummary) -> String {
    page_variance::page_audit_bucket(item.effective_total())
}
