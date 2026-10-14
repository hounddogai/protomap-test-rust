use super::*;

use std::collections::BTreeMap;

use crate::rollout::metric as rollout_metric;

pub fn summarize_rollout_metrics_for_window(items: &[rollout_metric::RolloutMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_freeze_windows_from_metric(
    items: &[rollout_metric::RolloutMetric],
) -> Vec<ScheduledFreezeWindow> {
    let mut out: Vec<ScheduledFreezeWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_freeze_window(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_scheduled_freeze_window(item: &ScheduledFreezeWindow) -> String {
    rollout_metric::classify_rollout_metric(item.effective_volume())
}
