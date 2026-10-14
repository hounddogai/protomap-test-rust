use super::*;

use std::collections::BTreeMap;

use crate::evidence::template as evidence_template;

pub fn summarize_evidence_templates_for_slot(items: &[evidence_template::EvidenceTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_measurement_slots_from_template(items: &[evidence_template::EvidenceTemplate]) -> Vec<MeasurementSlot> {
    let mut out: Vec<MeasurementSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_slot(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_measurement_slot(item: &MeasurementSlot) -> String {
    evidence_template::evidence_template_bucket(item.net_value())
}
