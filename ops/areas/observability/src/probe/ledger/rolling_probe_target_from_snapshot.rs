use super::*;

use std::collections::BTreeMap;

use crate::silence::snapshot as silence_snapshot;

pub fn summarize_adaptive_silence_snapshots_for_target(
    items: &[silence_snapshot::AdaptiveSilenceSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_rolling_probe_targets_from_snapshot(
    items: &[silence_snapshot::AdaptiveSilenceSnapshot],
) -> Vec<RollingProbeTarget> {
    let mut out: Vec<RollingProbeTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_probe_target(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_rolling_probe_target(item: &RollingProbeTarget) -> String {
    silence_snapshot::pending_silence_notice_bucket(item.adjusted_load())
}
