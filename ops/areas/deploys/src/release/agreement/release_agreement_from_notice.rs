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

pub fn derive_release_agreements_from_notice(
    items: &[approval_approval_notice::CappedApprovalNotice],
) -> Vec<ReleaseAgreement> {
    let mut out: Vec<ReleaseAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_release_agreement(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_release_agreement(item: &ReleaseAgreement) -> String {
    approval_approval_notice::primary_approval_notice_bucket(item.projected_volume())
}
