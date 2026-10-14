use super::*;

use std::collections::BTreeMap;

use crate::rollout::budget as rollout_budget;

pub fn summarize_rolling_rollout_snapshots_for_quota(
    items: &[rollout_budget::RollingRolloutSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rollout_quotas_from_snapshot(items: &[rollout_budget::RollingRolloutSnapshot]) -> Vec<RolloutQuota> {
    let mut out: Vec<RolloutQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_quota(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rollout_quota(item: &RolloutQuota) -> String {
    rollout_budget::scheduled_rollout_budget_bucket(item.adjusted_total())
}
