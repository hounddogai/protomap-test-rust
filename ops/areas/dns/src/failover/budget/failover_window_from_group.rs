use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::metric as ttl_policy_metric;

pub fn summarize_pending_ttl_policy_groups_for_window(
    items: &[ttl_policy_metric::PendingTtlPolicyGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_failover_windows_from_group(items: &[ttl_policy_metric::PendingTtlPolicyGroup]) -> Vec<FailoverWindow> {
    let mut out: Vec<FailoverWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_window(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_failover_window(item: &FailoverWindow) -> String {
    ttl_policy_metric::normalize_ttl_policy_entry_code(&item.code)
}
