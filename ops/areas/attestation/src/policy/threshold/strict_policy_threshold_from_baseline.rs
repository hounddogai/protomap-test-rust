use super::*;

use std::collections::BTreeMap;

use crate::quote::quote_claim as quote_quote_claim;

pub fn summarize_weighted_quote_baselines_for_threshold(
    items: &[quote_quote_claim::WeightedQuoteBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_strict_policy_thresholds_from_baseline(
    items: &[quote_quote_claim::WeightedQuoteBaseline],
) -> Vec<StrictPolicyThreshold> {
    let mut out: Vec<StrictPolicyThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_policy_threshold(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_strict_policy_threshold(item: &StrictPolicyThreshold) -> String {
    quote_quote_claim::tiered_quote_claim_bucket(item.projected_volume())
}
