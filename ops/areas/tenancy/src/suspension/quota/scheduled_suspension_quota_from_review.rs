use super::*;

use std::collections::BTreeMap;

use crate::isolation::review as isolation_review;

pub fn summarize_isolation_reviews_for_quota(items: &[isolation_review::IsolationReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_scheduled_suspension_quotas_from_review(
    items: &[isolation_review::IsolationReview],
) -> Vec<ScheduledSuspensionQuota> {
    let mut out: Vec<ScheduledSuspensionQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_suspension_quota(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_scheduled_suspension_quota(item: &ScheduledSuspensionQuota) -> String {
    isolation_review::isolation_summary_bucket(item.projected_load())
}
