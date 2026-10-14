use super::*;

use std::collections::BTreeMap;

use crate::grant::digest as grant_digest;

pub fn summarize_scheduled_grant_digests_for_review(
    items: &[grant_digest::ScheduledGrantDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_grant_reviews_from_digest(items: &[grant_digest::ScheduledGrantDigest]) -> Vec<GrantReview> {
    let mut out: Vec<GrantReview> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_review(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_grant_review(item: &GrantReview) -> String {
    grant_digest::grant_baseline_bucket(item.measured_value())
}
