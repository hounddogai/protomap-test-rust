use super::*;

use std::collections::BTreeMap;

use crate::suspension::allocation as suspension_allocation;

pub fn summarize_adaptive_suspension_quotas_for_preference(
    items: &[suspension_allocation::AdaptiveSuspensionQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_region_pin_preferences_from_quota(
    items: &[suspension_allocation::AdaptiveSuspensionQuota],
) -> Vec<RegionPinPreference> {
    let mut out: Vec<RegionPinPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_preference(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_region_pin_preference(item: &RegionPinPreference) -> f64 {
    suspension_allocation::suspension_allocation_weight(6, true)
}
