use super::*;

use std::collections::BTreeMap;

use crate::region_pin::summary as region_pin_summary;

pub fn summarize_rolling_region_pin_reviews_for_checkpoint(
    items: &[region_pin_summary::RollingRegionPinReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_rolling_plan_checkpoints_from_review(
    items: &[region_pin_summary::RollingRegionPinReview],
) -> Vec<RollingPlanCheckpoint> {
    let mut out: Vec<RollingPlanCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_plan_checkpoint(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_rolling_plan_checkpoint(item: &RollingPlanCheckpoint) -> f64 {
    region_pin_summary::fallback_region_pin_summary_weight(3, true)
}
