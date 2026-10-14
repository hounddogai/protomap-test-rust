use super::*;

use std::collections::BTreeMap;

use crate::postmortem::adjustment as postmortem_adjustment;

pub fn summarize_postmortem_reviews_for_score(
    items: &[postmortem_adjustment::PostmortemReview],
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

pub fn derive_rolling_postmortem_scores_from_review(
    items: &[postmortem_adjustment::PostmortemReview],
) -> Vec<RollingPostmortemScore> {
    let mut out: Vec<RollingPostmortemScore> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_postmortem_score(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_rolling_postmortem_score(item: &RollingPostmortemScore) -> String {
    postmortem_adjustment::classify_seasonal_postmortem_adjustment(item.adjusted_value())
}
