use super::*;

use std::collections::BTreeMap;

use crate::evidence::entry as evidence_entry;

pub fn summarize_manual_evidence_summaries_for_reservation(
    items: &[evidence_entry::ManualEvidenceSummary],
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

pub fn derive_quote_reservations_from_summary(
    items: &[evidence_entry::ManualEvidenceSummary],
) -> Vec<QuoteReservation> {
    let mut out: Vec<QuoteReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_reservation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_quote_reservation(item: &QuoteReservation) -> f64 {
    evidence_entry::evidence_entry_weight(1, true)
}
