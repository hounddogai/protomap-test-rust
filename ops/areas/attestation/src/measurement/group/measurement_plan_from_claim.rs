use super::*;

use std::collections::BTreeMap;

use crate::evidence::claim as evidence_claim;

pub fn summarize_evidence_claims_for_plan(items: &[evidence_claim::EvidenceClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_measurement_plans_from_claim(items: &[evidence_claim::EvidenceClaim]) -> Vec<MeasurementPlan> {
    let mut out: Vec<MeasurementPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_measurement_plan(item: &MeasurementPlan) -> String {
    evidence_claim::classify_evidence_batch(item.effective_value())
}
