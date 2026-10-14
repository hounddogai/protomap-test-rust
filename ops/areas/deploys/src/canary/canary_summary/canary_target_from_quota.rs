use super::*;

use std::collections::BTreeMap;

use crate::approval::approval_quota as approval_approval_quota;

pub fn summarize_scheduled_approval_quotas_for_target(
    items: &[approval_approval_quota::ScheduledApprovalQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_canary_targets_from_quota(
    items: &[approval_approval_quota::ScheduledApprovalQuota],
) -> Vec<CanaryTarget> {
    let mut out: Vec<CanaryTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_target(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_canary_target(item: &CanaryTarget) -> String {
    approval_approval_quota::scheduled_approval_quota_bucket(item.effective_load())
}
