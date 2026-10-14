use super::*;

use std::collections::BTreeMap;

use crate::health_check::cycle as health_check_cycle;

pub fn summarize_health_check_cycles_for_manifest(
    items: &[health_check_cycle::HealthCheckCycle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_batched_zone_manifests_from_cycle(
    items: &[health_check_cycle::HealthCheckCycle],
) -> Vec<BatchedZoneManifest> {
    let mut out: Vec<BatchedZoneManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_zone_manifest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_batched_zone_manifest(item: &BatchedZoneManifest) -> String {
    health_check_cycle::health_check_cycle_bucket(item.net_volume())
}
