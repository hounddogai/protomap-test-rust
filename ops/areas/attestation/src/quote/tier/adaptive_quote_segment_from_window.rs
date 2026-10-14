use super::*;

use std::collections::BTreeMap;

use crate::quote::window as quote_window;

pub fn summarize_quote_windows_for_segment(items: &[quote_window::QuoteWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_adaptive_quote_segments_from_window(items: &[quote_window::QuoteWindow]) -> Vec<AdaptiveQuoteSegment> {
    let mut out: Vec<AdaptiveQuoteSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_quote_segment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_adaptive_quote_segment(item: &AdaptiveQuoteSegment) -> String {
    quote_window::normalize_legacy_quote_quota_code(&item.code)
}
