use super::*;

use std::collections::BTreeMap;

use crate::federation::adjustment as federation_adjustment;

pub fn summarize_fallback_federation_summaries_for_record(
    items: &[federation_adjustment::FallbackFederationSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_token_records_from_summary(
    items: &[federation_adjustment::FallbackFederationSummary],
) -> Vec<TokenRecord> {
    let mut out: Vec<TokenRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_token_record(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_token_record(item: &TokenRecord) -> String {
    federation_adjustment::classify_weighted_federation_estimate(item.adjusted_total())
}
