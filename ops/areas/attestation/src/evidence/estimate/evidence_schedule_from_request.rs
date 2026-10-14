use super::*;

use std::collections::BTreeMap;

use crate::evidence::evidence_notice as evidence_evidence_notice;

pub fn summarize_rolling_evidence_requests_for_schedule(
    items: &[evidence_evidence_notice::RollingEvidenceRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_evidence_schedules_from_request(
    items: &[evidence_evidence_notice::RollingEvidenceRequest],
) -> Vec<EvidenceSchedule> {
    let mut out: Vec<EvidenceSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_schedule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_evidence_schedule(item: &EvidenceSchedule) -> String {
    evidence_evidence_notice::evidence_draft_bucket(item.weighted_total())
}
