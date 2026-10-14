use super::*;

use std::collections::BTreeMap;

use crate::measurement::profile as measurement_profile;

pub fn summarize_seasonal_measurement_manifests_for_slot(
    items: &[measurement_profile::SeasonalMeasurementManifest],
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

pub fn derive_nonce_slots_from_manifest(items: &[measurement_profile::SeasonalMeasurementManifest]) -> Vec<NonceSlot> {
    let mut out: Vec<NonceSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_slot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_nonce_slot(item: &NonceSlot) -> String {
    measurement_profile::seasonal_measurement_manifest_bucket(item.adjusted_exposure())
}
