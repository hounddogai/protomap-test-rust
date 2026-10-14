use super::*;

use std::collections::BTreeMap;

use crate::timeline::legacy_timeline_policy as timeline_legacy_timeline_policy;

pub fn summarize_strict_timeline_reviews_for_claim(
    items: &[timeline_legacy_timeline_policy::StrictTimelineReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_postmortem_claims_from_review(
    items: &[timeline_legacy_timeline_policy::StrictTimelineReview],
) -> Vec<PostmortemClaim> {
    let mut out: Vec<PostmortemClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_claim(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_postmortem_claim(item: &PostmortemClaim) -> String {
    timeline_legacy_timeline_policy::normalize_timeline_score_code(&item.code)
}
