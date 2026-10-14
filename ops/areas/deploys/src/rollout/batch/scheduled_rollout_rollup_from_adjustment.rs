use super::*;

use std::collections::BTreeMap;

use crate::rollback::template as rollback_template;

pub fn summarize_rollback_adjustments_for_rollup(
    items: &[rollback_template::RollbackAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_scheduled_rollout_rollups_from_adjustment(
    items: &[rollback_template::RollbackAdjustment],
) -> Vec<ScheduledRolloutRollup> {
    let mut out: Vec<ScheduledRolloutRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_rollout_rollup(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_scheduled_rollout_rollup(item: &ScheduledRolloutRollup) -> String {
    rollback_template::classify_rollback_adjustment(item.effective_value())
}
