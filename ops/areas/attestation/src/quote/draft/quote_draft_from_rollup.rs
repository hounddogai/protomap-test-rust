use super::*;

use std::collections::BTreeMap;

use crate::verifier::segment as verifier_segment;

pub fn summarize_verifier_rollups_for_draft(items: &[verifier_segment::VerifierRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_quote_drafts_from_rollup(items: &[verifier_segment::VerifierRollup]) -> Vec<QuoteDraft> {
    let mut out: Vec<QuoteDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_quote_draft(item: &QuoteDraft) -> f64 {
    verifier_segment::regional_verifier_segment_weight(3, true)
}
