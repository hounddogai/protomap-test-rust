use super::*;

use std::collections::BTreeMap;

use crate::plan::plan_variance as plan_plan_variance;

pub fn summarize_fallback_plan_items_for_review(
    items: &[plan_plan_variance::FallbackPlanItem],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_rolling_region_pin_reviews_from_item(
    items: &[plan_plan_variance::FallbackPlanItem],
) -> Vec<RollingRegionPinReview> {
    let mut out: Vec<RollingRegionPinReview> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_region_pin_review(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_rolling_region_pin_review_extra(item: &RollingRegionPinReview) -> String {
    plan_plan_variance::rolling_plan_mapping_bucket(item.net_total())
}
