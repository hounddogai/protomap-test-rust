use super::*;

use std::collections::BTreeMap;

use crate::rollback::rollback_snapshot as rollback_rollback_snapshot;

pub fn summarize_rolling_rollback_snapshots_for_policy(
    items: &[rollback_rollback_snapshot::RollingRollbackSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_scheduled_canary_policies_from_snapshot(
    items: &[rollback_rollback_snapshot::RollingRollbackSnapshot],
) -> Vec<ScheduledCanaryPolicy> {
    let mut out: Vec<ScheduledCanaryPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_canary_policy(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_scheduled_canary_policy(item: &ScheduledCanaryPolicy) -> String {
    rollback_rollback_snapshot::classify_rolling_rollback_snapshot(item.measured_value())
}
