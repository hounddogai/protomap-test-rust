use super::*;

use std::collections::BTreeMap;

use crate::policy::segment as policy_segment;

pub fn summarize_policy_audits_for_draft(items: &[policy_segment::PolicyAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_nonce_drafts_from_audit(items: &[policy_segment::PolicyAudit]) -> Vec<NonceDraft> {
    let mut out: Vec<NonceDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_draft(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_nonce_draft(item: &NonceDraft) -> String {
    policy_segment::policy_segment_bucket(item.measured_value())
}
