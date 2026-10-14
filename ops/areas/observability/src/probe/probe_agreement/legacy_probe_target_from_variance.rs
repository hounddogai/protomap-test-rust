use super::*;

use std::collections::BTreeMap;

use crate::silence::primary_silence_variance as silence_primary_silence_variance;

pub fn summarize_manual_silence_variances_for_target(
    items: &[silence_primary_silence_variance::ManualSilenceVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_legacy_probe_targets_from_variance(
    items: &[silence_primary_silence_variance::ManualSilenceVariance],
) -> Vec<LegacyProbeTarget> {
    let mut out: Vec<LegacyProbeTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_probe_target(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_legacy_probe_target(item: &LegacyProbeTarget) -> f64 {
    silence_primary_silence_variance::silence_segment_weight(3, true)
}
