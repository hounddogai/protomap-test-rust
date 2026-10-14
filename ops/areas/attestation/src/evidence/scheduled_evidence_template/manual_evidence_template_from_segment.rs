use super::*;

use std::collections::BTreeMap;

use crate::policy::schedule as policy_schedule;

pub fn summarize_strict_policy_segments_for_template(
    items: &[policy_schedule::StrictPolicySegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_manual_evidence_templates_from_segment(
    items: &[policy_schedule::StrictPolicySegment],
) -> Vec<ManualEvidenceTemplate> {
    let mut out: Vec<ManualEvidenceTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_evidence_template(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_manual_evidence_template(item: &ManualEvidenceTemplate) -> f64 {
    policy_schedule::strict_policy_segment_weight(1, true)
}
