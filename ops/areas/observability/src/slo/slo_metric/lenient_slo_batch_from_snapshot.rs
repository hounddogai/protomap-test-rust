use super::*;

use std::collections::BTreeMap;

use crate::silence::snapshot as silence_snapshot;

pub fn summarize_adaptive_silence_snapshots_for_batch(
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

pub fn derive_lenient_slo_batches_from_snapshot(
    items: &[silence_snapshot::AdaptiveSilenceSnapshot],
) -> Vec<LenientSloBatch> {
    let mut out: Vec<LenientSloBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_slo_batch(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_lenient_slo_batch(item: &LenientSloBatch) -> String {
    silence_snapshot::pending_silence_notice_bucket(item.net_total())
}
