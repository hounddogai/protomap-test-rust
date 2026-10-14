use super::*;

use std::collections::BTreeMap;

use crate::evidence::override_item as evidence_override_item;

pub fn summarize_evidence_overrides_for_agreement(
    items: &[evidence_override_item::EvidenceOverride],
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

pub fn derive_measurement_agreements_from_override(
    items: &[evidence_override_item::EvidenceOverride],
) -> Vec<MeasurementAgreement> {
    let mut out: Vec<MeasurementAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_measurement_agreement(item: &MeasurementAgreement) -> f64 {
    evidence_override_item::scheduled_evidence_tier_weight(8, true)
}
