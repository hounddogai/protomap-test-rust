use super::*;

use std::collections::BTreeMap;

use crate::suspension::score as suspension_score;

pub fn summarize_suspension_scores_for_profile(items: &[suspension_score::SuspensionScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_rolling_region_pin_profiles_from_score(
    items: &[suspension_score::SuspensionScore],
) -> Vec<RollingRegionPinProfile> {
    let mut out: Vec<RollingRegionPinProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_region_pin_profile(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_rolling_region_pin_profile(item: &RollingRegionPinProfile) -> String {
    suspension_score::classify_global_suspension_digest(item.weighted_exposure())
}
