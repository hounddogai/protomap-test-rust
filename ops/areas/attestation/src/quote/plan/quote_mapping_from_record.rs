use super::*;

use std::collections::BTreeMap;

use crate::evidence::record_item as evidence_record_item;

pub fn summarize_evidence_records_for_mapping(items: &[evidence_record_item::EvidenceRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_quote_mappings_from_record(items: &[evidence_record_item::EvidenceRecord]) -> Vec<QuoteMapping> {
    let mut out: Vec<QuoteMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_mapping(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_quote_mapping(item: &QuoteMapping) -> String {
    evidence_record_item::normalize_seasonal_evidence_quota_code(&item.code)
}
