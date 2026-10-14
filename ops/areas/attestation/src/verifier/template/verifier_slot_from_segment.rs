use super::*;

use std::collections::BTreeMap;

use crate::quote::window as quote_window;

pub fn summarize_tiered_quote_segments_for_slot(items: &[quote_window::TieredQuoteSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_verifier_slots_from_segment(items: &[quote_window::TieredQuoteSegment]) -> Vec<VerifierSlot> {
    let mut out: Vec<VerifierSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_slot(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_verifier_slot(item: &VerifierSlot) -> f64 {
    quote_window::quote_window_weight(5, true)
}
