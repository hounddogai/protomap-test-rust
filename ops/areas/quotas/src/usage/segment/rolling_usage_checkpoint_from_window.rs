use super::*;

use std::collections::BTreeMap;

use crate::grant::targets as grant_targets;

pub fn summarize_seasonal_grant_windows_for_checkpoint(
    items: &[grant_targets::SeasonalGrantWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_rolling_usage_checkpoints_from_window(
    items: &[grant_targets::SeasonalGrantWindow],
) -> Vec<RollingUsageCheckpoint> {
    let mut out: Vec<RollingUsageCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_usage_checkpoint(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_rolling_usage_checkpoint(item: &RollingUsageCheckpoint) -> String {
    grant_targets::normalize_capped_grant_target_code(&item.code)
}
