use super::*;

use std::collections::BTreeMap;

use crate::silence::metric as silence_metric;

pub fn summarize_pending_silence_estimates_for_snapshot(
    items: &[silence_metric::PendingSilenceEstimate],
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

pub fn derive_probe_snapshots_from_estimate(items: &[silence_metric::PendingSilenceEstimate]) -> Vec<ProbeSnapshot> {
    let mut out: Vec<ProbeSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_snapshot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_probe_snapshot(item: &ProbeSnapshot) -> String {
    silence_metric::classify_manual_silence_cycle(item.effective_exposure())
}
