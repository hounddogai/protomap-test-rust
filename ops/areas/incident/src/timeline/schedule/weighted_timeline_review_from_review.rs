use super::*;

use std::collections::BTreeMap;

use crate::postmortem::postmortem_manifest as postmortem_postmortem_manifest;

pub fn summarize_pending_postmortem_reviews_for_review(
    items: &[postmortem_postmortem_manifest::PendingPostmortemReview],
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

pub fn derive_weighted_timeline_reviews_from_review(
    items: &[postmortem_postmortem_manifest::PendingPostmortemReview],
) -> Vec<WeightedTimelineReview> {
    let mut out: Vec<WeightedTimelineReview> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_timeline_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_weighted_timeline_review(item: &WeightedTimelineReview) -> String {
    postmortem_postmortem_manifest::classify_seasonal_postmortem_manifest(item.projected_exposure())
}
