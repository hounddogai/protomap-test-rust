use super::*;

use std::collections::BTreeMap;

use crate::approval::rule as approval_rule;

pub fn summarize_legacy_approval_checkpoints_for_agreement(
    items: &[approval_rule::LegacyApprovalCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_rolling_approval_agreements_from_checkpoint(
    items: &[approval_rule::LegacyApprovalCheckpoint],
) -> Vec<RollingApprovalAgreement> {
    let mut out: Vec<RollingApprovalAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_approval_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_rolling_approval_agreement(item: &RollingApprovalAgreement) -> String {
    approval_rule::strict_approval_threshold_bucket(item.weighted_total())
}
