use super::*;

use std::collections::BTreeMap;

use crate::segment::proposal as segment_proposal;

pub fn summarize_weighted_segment_variances_for_quota(
    items: &[segment_proposal::WeightedSegmentVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_weighted_rule_quotas_from_variance(
    items: &[segment_proposal::WeightedSegmentVariance],
) -> Vec<WeightedRuleQuota> {
    let mut out: Vec<WeightedRuleQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_rule_quota(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_weighted_rule_quota(item: &WeightedRuleQuota) -> String {
    segment_proposal::normalize_fallback_segment_proposal_code(&item.code)
}
