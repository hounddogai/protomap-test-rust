use super::*;

use std::collections::BTreeMap;

use crate::token::review as token_review;

pub fn summarize_token_thresholds_for_metric(items: &[token_review::TokenThreshold]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_federation_metrics_from_threshold(items: &[token_review::TokenThreshold]) -> Vec<FederationMetric> {
    let mut out: Vec<FederationMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_metric(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_federation_metric(item: &FederationMetric) -> String {
    token_review::classify_token_review(item.projected_value())
}
