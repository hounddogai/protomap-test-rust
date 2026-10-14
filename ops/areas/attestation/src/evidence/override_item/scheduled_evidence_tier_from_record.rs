use super::*;

use std::collections::BTreeMap;

use crate::evidence::record_item as evidence_record_item;

pub fn summarize_evidence_records_for_tier(items: &[evidence_record_item::EvidenceRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_evidence_tiers_from_record(
    items: &[evidence_record_item::EvidenceRecord],
) -> Vec<ScheduledEvidenceTier> {
    let mut out: Vec<ScheduledEvidenceTier> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_evidence_tier(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_scheduled_evidence_tier(item: &ScheduledEvidenceTier) -> String {
    evidence_record_item::evidence_record_bucket(item.projected_value())
}
