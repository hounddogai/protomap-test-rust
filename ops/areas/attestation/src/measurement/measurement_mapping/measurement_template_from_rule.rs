use super::*;

use std::collections::BTreeMap;

use crate::evidence::scheduled_evidence_template as evidence_scheduled_evidence_template;

pub fn summarize_evidence_rules_for_template(
    items: &[evidence_scheduled_evidence_template::EvidenceRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_measurement_templates_from_rule(
    items: &[evidence_scheduled_evidence_template::EvidenceRule],
) -> Vec<MeasurementTemplate> {
    let mut out: Vec<MeasurementTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_template(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}
