use super::*;

use std::collections::BTreeMap;

use crate::rollback::baseline as rollback_baseline;

pub fn summarize_rollback_preferences_for_profile(
    items: &[rollback_baseline::RollbackPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_freeze_profiles_from_preference(items: &[rollback_baseline::RollbackPreference]) -> Vec<FreezeProfile> {
    let mut out: Vec<FreezeProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_profile(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_freeze_profile(item: &FreezeProfile) -> f64 {
    rollback_baseline::rollback_preference_weight(2, true)
}
