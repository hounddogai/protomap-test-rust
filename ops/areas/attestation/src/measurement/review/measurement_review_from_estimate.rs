use super::*;

use std::collections::BTreeMap;

use crate::quote::batch as quote_batch;

pub fn summarize_rolling_quote_estimates_for_review(
    items: &[quote_batch::RollingQuoteEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_measurement_reviews_from_estimate(items: &[quote_batch::RollingQuoteEstimate]) -> Vec<MeasurementReview> {
    let mut out: Vec<MeasurementReview> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_review(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_measurement_review(item: &MeasurementReview) -> String {
    quote_batch::normalize_rolling_quote_estimate_code(&item.code)
}
