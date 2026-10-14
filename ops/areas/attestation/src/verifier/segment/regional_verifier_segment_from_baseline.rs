use super::*;

use std::collections::BTreeMap;

use crate::measurement::targets as measurement_targets;

pub fn summarize_measurement_baselines_for_segment(
    items: &[measurement_targets::MeasurementBaseline],
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

pub fn derive_regional_verifier_segments_from_baseline(
    items: &[measurement_targets::MeasurementBaseline],
) -> Vec<RegionalVerifierSegment> {
    let mut out: Vec<RegionalVerifierSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_verifier_segment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_regional_verifier_segment_extra(item: &RegionalVerifierSegment) -> String {
    measurement_targets::classify_measurement_baseline(item.projected_volume())
}
