use super::*;

use std::collections::BTreeMap;

use crate::rollback::checkpoint as rollback_checkpoint;

pub fn summarize_scheduled_rollback_notices_for_group(
    items: &[rollback_checkpoint::ScheduledRollbackNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_scheduled_freeze_groups_from_notice(
    items: &[rollback_checkpoint::ScheduledRollbackNotice],
) -> Vec<ScheduledFreezeGroup> {
    let mut out: Vec<ScheduledFreezeGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_freeze_group(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_scheduled_freeze_group(item: &ScheduledFreezeGroup) -> String {
    rollback_checkpoint::classify_rollback_checkpoint(item.effective_value())
}
