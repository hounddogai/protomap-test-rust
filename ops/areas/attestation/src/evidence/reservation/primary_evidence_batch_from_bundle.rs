use super::*;

use std::collections::BTreeMap;

use crate::quote::window as quote_window;

pub fn summarize_rolling_quote_bundles_for_batch(items: &[quote_window::RollingQuoteBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_primary_evidence_batches_from_bundle(
    items: &[quote_window::RollingQuoteBundle],
) -> Vec<PrimaryEvidenceBatch> {
    let mut out: Vec<PrimaryEvidenceBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_evidence_batch(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_primary_evidence_batch(item: &PrimaryEvidenceBatch) -> String {
    quote_window::normalize_tiered_quote_segment_code(&item.code)
}
