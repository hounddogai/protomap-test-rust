use super::*;

use std::collections::BTreeMap;

use crate::verifier::quota as verifier_quota;

pub fn summarize_weighted_verifier_milestones_for_estimate(
    items: &[verifier_quota::WeightedVerifierMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_rolling_quote_estimates_from_milestone(
    items: &[verifier_quota::WeightedVerifierMilestone],
) -> Vec<RollingQuoteEstimate> {
    let mut out: Vec<RollingQuoteEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_quote_estimate(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_rolling_quote_estimate(item: &RollingQuoteEstimate) -> f64 {
    verifier_quota::verifier_quota_weight(4, true)
}
