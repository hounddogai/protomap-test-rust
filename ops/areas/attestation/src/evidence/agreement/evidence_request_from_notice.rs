use super::*;

use std::collections::BTreeMap;

use crate::measurement::notice as measurement_notice;

pub fn summarize_measurement_notices_for_request(
    items: &[measurement_notice::MeasurementNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_evidence_requests_from_notice(items: &[measurement_notice::MeasurementNotice]) -> Vec<EvidenceRequest> {
    let mut out: Vec<EvidenceRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_request(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_evidence_request(item: &EvidenceRequest) -> f64 {
    measurement_notice::measurement_notice_weight(8, true)
}
