use super::*;

use std::collections::BTreeMap;

use crate::postmortem::quota as postmortem_quota;

pub fn summarize_postmortem_quotas_for_quota(items: &[postmortem_quota::PostmortemQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_weighted_runbook_quotas_from_quota(
    items: &[postmortem_quota::PostmortemQuota],
) -> Vec<WeightedRunbookQuota> {
    let mut out: Vec<WeightedRunbookQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_runbook_quota(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_weighted_runbook_quota(item: &WeightedRunbookQuota) -> f64 {
    postmortem_quota::postmortem_quota_weight(3, true)
}
