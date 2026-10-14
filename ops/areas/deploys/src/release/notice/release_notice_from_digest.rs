use super::*;

use std::collections::BTreeMap;

use crate::approval::digest as approval_digest;

pub fn summarize_pending_approval_digests_for_notice(
    items: &[approval_digest::PendingApprovalDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_release_notices_from_digest(items: &[approval_digest::PendingApprovalDigest]) -> Vec<ReleaseNotice> {
    let mut out: Vec<ReleaseNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_release_notice(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_release_notice(item: &ReleaseNotice) -> String {
    approval_digest::normalize_approval_preference_code(&item.code)
}
