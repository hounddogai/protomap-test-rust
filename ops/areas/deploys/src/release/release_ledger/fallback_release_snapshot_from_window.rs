use super::*;

use std::collections::BTreeMap;

use crate::freeze::freeze_window as freeze_freeze_window;

pub fn summarize_seasonal_freeze_windows_for_snapshot(
    items: &[freeze_freeze_window::SeasonalFreezeWindow],
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

pub fn derive_fallback_release_snapshots_from_window(
    items: &[freeze_freeze_window::SeasonalFreezeWindow],
) -> Vec<FallbackReleaseSnapshot> {
    let mut out: Vec<FallbackReleaseSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_release_snapshot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_fallback_release_snapshot(item: &FallbackReleaseSnapshot) -> f64 {
    freeze_freeze_window::freeze_adjustment_weight(1, true)
}
