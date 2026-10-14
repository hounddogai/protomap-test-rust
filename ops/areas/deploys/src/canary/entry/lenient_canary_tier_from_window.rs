use super::*;

use std::collections::BTreeMap;

use crate::rollout::rollout_window as rollout_rollout_window;

pub fn summarize_legacy_rollout_windows_for_tier(
    items: &[rollout_rollout_window::LegacyRolloutWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_canary_tiers_from_window(
    items: &[rollout_rollout_window::LegacyRolloutWindow],
) -> Vec<LenientCanaryTier> {
    let mut out: Vec<LenientCanaryTier> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_canary_tier(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_lenient_canary_tier(item: &LenientCanaryTier) -> String {
    rollout_rollout_window::classify_rollout_score(item.net_value())
}
