use super::*;

use std::collections::BTreeMap;

use crate::failover::entry as failover_entry;

pub fn summarize_regional_failover_checkpoints_for_digest(
    items: &[failover_entry::RegionalFailoverCheckpoint],
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

pub fn derive_global_zone_digests_from_checkpoint(
    items: &[failover_entry::RegionalFailoverCheckpoint],
) -> Vec<GlobalZoneDigest> {
    let mut out: Vec<GlobalZoneDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_global_zone_digest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_global_zone_digest(item: &GlobalZoneDigest) -> f64 {
    failover_entry::regional_failover_override_weight(5, true)
}
