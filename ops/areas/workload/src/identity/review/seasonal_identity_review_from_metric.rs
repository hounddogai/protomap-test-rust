use super::*;

use std::collections::BTreeMap;

use crate::token::metric as token_metric;

pub fn summarize_token_metrics_for_review(items: &[token_metric::TokenMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_seasonal_identity_reviews_from_metric(
    items: &[token_metric::TokenMetric],
) -> Vec<SeasonalIdentityReview> {
    let mut out: Vec<SeasonalIdentityReview> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_identity_review(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_seasonal_identity_review(item: &SeasonalIdentityReview) -> String {
    token_metric::token_notice_bucket(item.measured_value())
}
