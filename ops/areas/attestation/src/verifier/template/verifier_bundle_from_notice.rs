use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_notice as measurement_measurement_notice;

pub fn summarize_legacy_measurement_notices_for_bundle(
    items: &[measurement_measurement_notice::LegacyMeasurementNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_verifier_bundles_from_notice(
    items: &[measurement_measurement_notice::LegacyMeasurementNotice],
) -> Vec<VerifierBundle> {
    let mut out: Vec<VerifierBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_bundle(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_verifier_bundle(item: &VerifierBundle) -> String {
    measurement_measurement_notice::normalize_measurement_digest_code(&item.code)
}
