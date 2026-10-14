use super::*;

use std::collections::BTreeMap;

use crate::nonce::policy as nonce_policy;

pub fn summarize_lenient_nonce_metrics_for_audit(items: &[nonce_policy::LenientNonceMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_evidence_audits_from_metric(items: &[nonce_policy::LenientNonceMetric]) -> Vec<EvidenceAudit> {
    let mut out: Vec<EvidenceAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_audit(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_evidence_audit(item: &EvidenceAudit) -> String {
    nonce_policy::nonce_policy_bucket(item.net_value())
}
