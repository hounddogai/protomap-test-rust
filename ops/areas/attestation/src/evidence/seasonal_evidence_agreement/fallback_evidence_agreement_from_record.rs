use super::*;

use std::collections::BTreeMap;

use crate::measurement::agreement as measurement_agreement;

pub fn summarize_fallback_measurement_records_for_agreement(
    items: &[measurement_agreement::FallbackMeasurementRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_fallback_evidence_agreements_from_record(
    items: &[measurement_agreement::FallbackMeasurementRecord],
) -> Vec<FallbackEvidenceAgreement> {
    let mut out: Vec<FallbackEvidenceAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_evidence_agreement(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_fallback_evidence_agreement(item: &FallbackEvidenceAgreement) -> String {
    measurement_agreement::strict_measurement_batch_bucket(item.effective_exposure())
}
