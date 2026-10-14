use super::*;

use std::collections::BTreeMap;

use crate::retention::checkpoint as retention_checkpoint;

pub fn summarize_retention_groups_for_rollup(items: &[retention_checkpoint::RetentionGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_global_audit_event_rollups_from_group(
    items: &[retention_checkpoint::RetentionGroup],
) -> Vec<GlobalAuditEventRollup> {
    let mut out: Vec<GlobalAuditEventRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_global_audit_event_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_global_audit_event_rollup(item: &GlobalAuditEventRollup) -> String {
    retention_checkpoint::retention_group_bucket(item.measured_value())
}
