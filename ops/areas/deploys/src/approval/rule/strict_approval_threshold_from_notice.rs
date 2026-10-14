use super::*;

use std::collections::BTreeMap;

use crate::rollback::checkpoint as rollback_checkpoint;

pub fn summarize_scheduled_rollback_notices_for_threshold(
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

pub fn derive_strict_approval_thresholds_from_notice(
    items: &[rollback_checkpoint::ScheduledRollbackNotice],
) -> Vec<StrictApprovalThreshold> {
    let mut out: Vec<StrictApprovalThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_approval_threshold(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_strict_approval_threshold(item: &StrictApprovalThreshold) -> String {
    rollback_checkpoint::classify_rollback_checkpoint(item.measured_value())
}
