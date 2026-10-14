use super::*;

use std::collections::BTreeMap;

use crate::policy::segment as policy_segment;

pub fn summarize_policy_audits_for_summary(items: &[policy_segment::PolicyAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_verifier_summaries_from_audit(items: &[policy_segment::PolicyAudit]) -> Vec<VerifierSummary> {
    let mut out: Vec<VerifierSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_summary(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_verifier_summary(item: &VerifierSummary) -> String {
    policy_segment::classify_policy_quota(item.weighted_volume())
}
