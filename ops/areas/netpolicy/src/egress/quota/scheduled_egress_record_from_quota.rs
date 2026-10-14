use super::*;

use std::collections::BTreeMap;

use crate::allowlist::allowlist_assignment as allowlist_allowlist_assignment;

pub fn summarize_batched_allowlist_quotas_for_record(
    items: &[allowlist_allowlist_assignment::BatchedAllowlistQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_scheduled_egress_records_from_quota(
    items: &[allowlist_allowlist_assignment::BatchedAllowlistQuota],
) -> Vec<ScheduledEgressRecord> {
    let mut out: Vec<ScheduledEgressRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_egress_record(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_scheduled_egress_record(item: &ScheduledEgressRecord) -> f64 {
    allowlist_allowlist_assignment::lenient_allowlist_budget_weight(9, true)
}
