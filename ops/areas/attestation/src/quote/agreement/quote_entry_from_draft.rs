use super::*;

use std::collections::BTreeMap;

use crate::nonce::draft as nonce_draft;

pub fn summarize_nonce_drafts_for_entry(items: &[nonce_draft::NonceDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_quote_entries_from_draft(items: &[nonce_draft::NonceDraft]) -> Vec<QuoteEntry> {
    let mut out: Vec<QuoteEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_entry(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_quote_entry(item: &QuoteEntry) -> String {
    nonce_draft::classify_lenient_nonce_baseline(item.projected_load())
}
