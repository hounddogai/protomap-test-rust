use super::*;

use std::collections::BTreeMap;

use crate::retention::budget as retention_budget;

pub fn summarize_legacy_retention_reviews_for_tier(
    items: &[retention_budget::LegacyRetentionReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_regional_retention_tiers_from_review(
    items: &[retention_budget::LegacyRetentionReview],
) -> Vec<RegionalRetentionTier> {
    let mut out: Vec<RegionalRetentionTier> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_retention_tier(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_regional_retention_tier(item: &RegionalRetentionTier) -> f64 {
    retention_budget::weighted_retention_budget_weight(7, true)
}
