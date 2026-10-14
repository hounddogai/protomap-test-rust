use super::*;

use std::collections::BTreeMap;

use crate::release::window as release_window;

pub fn summarize_release_windows_for_rollup(items: &[release_window::ReleaseWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_default_rollout_rollups_from_window(
    items: &[release_window::ReleaseWindow],
) -> Vec<DefaultRolloutRollup> {
    let mut out: Vec<DefaultRolloutRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_default_rollout_rollup(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_default_rollout_rollup(item: &DefaultRolloutRollup) -> String {
    release_window::classify_release_window(item.net_volume())
}
