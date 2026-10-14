use super::*;

use std::collections::BTreeMap;

use crate::evidence::evidence_agreement as evidence_evidence_agreement;

pub fn summarize_evidence_notices_for_threshold(
    items: &[evidence_evidence_agreement::EvidenceNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_policy_thresholds_from_notice(
    items: &[evidence_evidence_agreement::EvidenceNotice],
) -> Vec<PolicyThreshold> {
    let mut out: Vec<PolicyThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_threshold(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_policy_threshold(item: &PolicyThreshold) -> String {
    evidence_evidence_agreement::normalize_manual_evidence_agreement_code(&item.code)
}
