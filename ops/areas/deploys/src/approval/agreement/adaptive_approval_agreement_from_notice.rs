use super::*;

use std::collections::BTreeMap;

use crate::approval::approval_notice as approval_approval_notice;

pub fn summarize_capped_approval_notices_for_agreement(
    items: &[approval_approval_notice::CappedApprovalNotice],
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

pub fn derive_adaptive_approval_agreements_from_notice(
    items: &[approval_approval_notice::CappedApprovalNotice],
) -> Vec<AdaptiveApprovalAgreement> {
    let mut out: Vec<AdaptiveApprovalAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_approval_agreement(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_adaptive_approval_agreement(item: &AdaptiveApprovalAgreement) -> f64 {
    approval_approval_notice::capped_approval_notice_weight(2, true)
}
