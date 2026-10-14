use super::*;

use std::collections::BTreeMap;

use crate::failover::preference as failover_preference;

pub fn summarize_failover_milestones_for_review(
    items: &[failover_preference::FailoverMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_lenient_zone_reviews_from_milestone(
    items: &[failover_preference::FailoverMilestone],
) -> Vec<LenientZoneReview> {
    let mut out: Vec<LenientZoneReview> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_zone_review(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_lenient_zone_review(item: &LenientZoneReview) -> String {
    failover_preference::failover_reservation_bucket(item.net_total())
}
