use super::*;

use std::collections::BTreeMap;

use crate::quote::record_item as quote_record_item;

pub fn summarize_batched_quote_digests_for_template(
    items: &[quote_record_item::BatchedQuoteDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_capped_evidence_templates_from_digest(
    items: &[quote_record_item::BatchedQuoteDigest],
) -> Vec<CappedEvidenceTemplate> {
    let mut out: Vec<CappedEvidenceTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_evidence_template(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_capped_evidence_template(item: &CappedEvidenceTemplate) -> String {
    quote_record_item::normalize_scheduled_quote_segment_code(&item.code)
}
