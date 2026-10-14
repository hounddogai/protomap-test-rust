use super::*;

use std::collections::BTreeMap;

use crate::verifier::baseline as verifier_baseline;

pub fn summarize_verifier_milestones_for_notice(
    items: &[verifier_baseline::VerifierMilestone],
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

pub fn derive_legacy_measurement_notices_from_milestone(
    items: &[verifier_baseline::VerifierMilestone],
) -> Vec<LegacyMeasurementNotice> {
    let mut out: Vec<LegacyMeasurementNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_measurement_notice(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_legacy_measurement_notice(item: &LegacyMeasurementNotice) -> String {
    verifier_baseline::classify_capped_verifier_baseline(item.net_volume())
}
