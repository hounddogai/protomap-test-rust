use super::*;

use std::collections::BTreeMap;

use crate::escalation::escalation_tier as escalation_escalation_tier;

pub fn summarize_weighted_escalation_tiers_for_review(
    items: &[escalation_escalation_tier::WeightedEscalationTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_strict_timeline_reviews_from_tier(
    items: &[escalation_escalation_tier::WeightedEscalationTier],
) -> Vec<StrictTimelineReview> {
    let mut out: Vec<StrictTimelineReview> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_timeline_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_strict_timeline_review(item: &StrictTimelineReview) -> String {
    escalation_escalation_tier::normalize_primary_escalation_notice_code(&item.code)
}
