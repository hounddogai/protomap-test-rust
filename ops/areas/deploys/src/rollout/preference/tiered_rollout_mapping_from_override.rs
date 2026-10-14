use super::*;

use std::collections::BTreeMap;

use crate::freeze::exception as freeze_exception;

pub fn summarize_freeze_overrides_for_mapping(items: &[freeze_exception::FreezeOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_tiered_rollout_mappings_from_override(
    items: &[freeze_exception::FreezeOverride],
) -> Vec<TieredRolloutMapping> {
    let mut out: Vec<TieredRolloutMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_rollout_mapping(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_tiered_rollout_mapping(item: &TieredRolloutMapping) -> String {
    freeze_exception::normalize_freeze_exception_code(&item.code)
}
