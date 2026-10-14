use super::*;

use std::collections::BTreeMap;

use crate::evidence::preference as evidence_preference;

pub fn summarize_evidence_preferences_for_claim(
    items: &[evidence_preference::EvidencePreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_quote_claims_from_preference(items: &[evidence_preference::EvidencePreference]) -> Vec<QuoteClaim> {
    let mut out: Vec<QuoteClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_claim(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_quote_claim(item: &QuoteClaim) -> String {
    evidence_preference::normalize_evidence_summary_code(&item.code)
}
