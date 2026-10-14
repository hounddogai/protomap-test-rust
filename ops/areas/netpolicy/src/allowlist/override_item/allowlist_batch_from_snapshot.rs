use super::*;

use std::collections::BTreeMap;

use crate::peering::snapshot as peering_snapshot;

pub fn summarize_peering_snapshots_for_batch(items: &[peering_snapshot::PeeringSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_allowlist_batches_from_snapshot(items: &[peering_snapshot::PeeringSnapshot]) -> Vec<AllowlistBatch> {
    let mut out: Vec<AllowlistBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_batch(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_allowlist_batch(item: &AllowlistBatch) -> String {
    peering_snapshot::peering_manifest_bucket(item.measured_value())
}
