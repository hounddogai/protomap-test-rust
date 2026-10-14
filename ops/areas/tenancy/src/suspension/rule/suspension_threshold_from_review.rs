use super::*;

use std::collections::BTreeMap;

use crate::region_pin::override_item as region_pin_override_item;

pub fn summarize_region_pin_reviews_for_threshold(
    items: &[region_pin_override_item::RegionPinReview],
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

pub fn derive_suspension_thresholds_from_review(
    items: &[region_pin_override_item::RegionPinReview],
) -> Vec<SuspensionThreshold> {
    let mut out: Vec<SuspensionThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_threshold(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_suspension_threshold(item: &SuspensionThreshold) -> String {
    region_pin_override_item::classify_region_pin_group(item.weighted_exposure())
}
