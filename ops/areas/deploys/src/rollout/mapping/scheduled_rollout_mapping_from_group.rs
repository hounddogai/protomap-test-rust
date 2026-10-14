use super::*;

use std::collections::BTreeMap;

use crate::freeze::schedule as freeze_schedule;

pub fn summarize_tiered_freeze_groups_for_mapping(
    items: &[freeze_schedule::TieredFreezeGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_scheduled_rollout_mappings_from_group(
    items: &[freeze_schedule::TieredFreezeGroup],
) -> Vec<ScheduledRolloutMapping> {
    let mut out: Vec<ScheduledRolloutMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_rollout_mapping(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_scheduled_rollout_mapping(item: &ScheduledRolloutMapping) -> f64 {
    freeze_schedule::freeze_checkpoint_weight(7, true)
}
