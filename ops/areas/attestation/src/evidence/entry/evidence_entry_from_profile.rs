use super::*;

use std::collections::BTreeMap;

use crate::quote::manual_quote_plan as quote_manual_quote_plan;

pub fn summarize_lenient_quote_profiles_for_entry(
    items: &[quote_manual_quote_plan::LenientQuoteProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_evidence_entries_from_profile(
    items: &[quote_manual_quote_plan::LenientQuoteProfile],
) -> Vec<EvidenceEntry> {
    let mut out: Vec<EvidenceEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_evidence_entry(item: &EvidenceEntry) -> f64 {
    quote_manual_quote_plan::legacy_quote_group_weight(3, true)
}
