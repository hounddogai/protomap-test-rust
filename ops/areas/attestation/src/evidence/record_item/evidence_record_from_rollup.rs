use super::*;

use std::collections::BTreeMap;

use crate::policy::policy_snapshot as policy_policy_snapshot;

pub fn summarize_primary_policy_rollups_for_record(
    items: &[policy_policy_snapshot::PrimaryPolicyRollup],
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

pub fn derive_evidence_records_from_rollup(
    items: &[policy_policy_snapshot::PrimaryPolicyRollup],
) -> Vec<EvidenceRecord> {
    let mut out: Vec<EvidenceRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_record(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_evidence_record(item: &EvidenceRecord) -> String {
    policy_policy_snapshot::classify_seasonal_policy_snapshot(item.measured_value())
}
