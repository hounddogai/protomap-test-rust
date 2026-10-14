use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_notice as measurement_measurement_notice;

pub fn summarize_measurement_rules_for_notice(
    items: &[measurement_measurement_notice::MeasurementRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_evidence_notices_from_rule(
    items: &[measurement_measurement_notice::MeasurementRule],
) -> Vec<EvidenceNotice> {
    let mut out: Vec<EvidenceNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_notice(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_evidence_notice(item: &EvidenceNotice) -> String {
    measurement_measurement_notice::classify_measurement_rule(item.projected_value())
}
