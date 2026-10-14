use super::*;

use std::collections::BTreeMap;

use crate::evidence::milestone as evidence_milestone;

pub fn summarize_evidence_rollups_for_entry(items: &[evidence_milestone::EvidenceRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_primary_evidence_entries_from_rollup(
    items: &[evidence_milestone::EvidenceRollup],
) -> Vec<PrimaryEvidenceEntry> {
    let mut out: Vec<PrimaryEvidenceEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_evidence_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_primary_evidence_entry(item: &PrimaryEvidenceEntry) -> f64 {
    evidence_milestone::legacy_evidence_request_weight(5, true)
}
