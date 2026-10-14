use super::*;

use std::collections::BTreeMap;

use crate::usage::usage_batch as usage_usage_batch;

pub fn summarize_usage_reviews_for_quota(items: &[usage_usage_batch::UsageReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_pending_usage_quotas_from_review(items: &[usage_usage_batch::UsageReview]) -> Vec<PendingUsageQuota> {
    let mut out: Vec<PendingUsageQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_usage_quota(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_pending_usage_quota(item: &PendingUsageQuota) -> String {
    usage_usage_batch::classify_usage_review(item.net_value())
}
