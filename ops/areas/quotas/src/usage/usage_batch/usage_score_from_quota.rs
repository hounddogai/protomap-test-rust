use super::*;

use std::collections::BTreeMap;

use crate::overage::quota as overage_quota;

pub fn summarize_capped_overage_quotas_for_score(items: &[overage_quota::CappedOverageQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_usage_scores_from_quota(items: &[overage_quota::CappedOverageQuota]) -> Vec<UsageScore> {
    let mut out: Vec<UsageScore> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_score(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_usage_score(item: &UsageScore) -> String {
    overage_quota::normalize_overage_budget_code(&item.code)
}
