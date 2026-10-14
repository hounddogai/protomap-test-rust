use super::*;

use std::collections::BTreeMap;

use crate::usage::manifest as usage_manifest;

pub fn summarize_usage_reservations_for_bundle(items: &[usage_manifest::UsageReservation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_strict_overage_bundles_from_reservation(
    items: &[usage_manifest::UsageReservation],
) -> Vec<StrictOverageBundle> {
    let mut out: Vec<StrictOverageBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_overage_bundle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_strict_overage_bundle(item: &StrictOverageBundle) -> String {
    usage_manifest::rolling_usage_manifest_bucket(item.weighted_volume())
}
