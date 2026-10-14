use super::*;

use std::collections::BTreeMap;

use crate::evidence::plan as evidence_plan;

pub fn summarize_legacy_evidence_assignments_for_mapping(
    items: &[evidence_plan::LegacyEvidenceAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_policy_mappings_from_assignment(items: &[evidence_plan::LegacyEvidenceAssignment]) -> Vec<PolicyMapping> {
    let mut out: Vec<PolicyMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_mapping(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_policy_mapping(item: &PolicyMapping) -> String {
    evidence_plan::normalize_primary_evidence_plan_code(&item.code)
}
