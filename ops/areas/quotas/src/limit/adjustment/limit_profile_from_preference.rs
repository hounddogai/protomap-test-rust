use super::*;

use std::collections::BTreeMap;

use crate::overage::slot as overage_slot;

pub fn summarize_primary_overage_preferences_for_profile(
    items: &[overage_slot::PrimaryOveragePreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_limit_profiles_from_preference(items: &[overage_slot::PrimaryOveragePreference]) -> Vec<LimitProfile> {
    let mut out: Vec<LimitProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_profile(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_limit_profile(item: &LimitProfile) -> String {
    overage_slot::normalize_overage_slot_code(&item.code)
}
