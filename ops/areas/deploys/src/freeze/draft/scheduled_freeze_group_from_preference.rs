use super::*;

use std::collections::BTreeMap;

use crate::rollout::rollout_preference as rollout_rollout_preference;

pub fn summarize_rolling_rollout_preferences_for_group(
    items: &[rollout_rollout_preference::RollingRolloutPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_scheduled_freeze_groups_from_preference(
    items: &[rollout_rollout_preference::RollingRolloutPreference],
) -> Vec<ScheduledFreezeGroup> {
    let mut out: Vec<ScheduledFreezeGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_freeze_group(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_scheduled_freeze_group(item: &ScheduledFreezeGroup) -> String {
    rollout_rollout_preference::rollout_agreement_bucket(item.effective_value())
}
