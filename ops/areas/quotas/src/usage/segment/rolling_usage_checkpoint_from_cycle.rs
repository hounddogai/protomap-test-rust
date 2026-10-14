use super::*;

use std::collections::BTreeMap;

use crate::usage::metric as usage_metric;

pub fn summarize_usage_cycles_for_checkpoint(items: &[usage_metric::UsageCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_rolling_usage_checkpoints_from_cycle(items: &[usage_metric::UsageCycle]) -> Vec<RollingUsageCheckpoint> {
    let mut out: Vec<RollingUsageCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_usage_checkpoint(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_rolling_usage_checkpoint(item: &RollingUsageCheckpoint) -> String {
    usage_metric::global_usage_metric_bucket(item.effective_value())
}
