use super::*;

use std::collections::BTreeMap;

use crate::rollback::checkpoint as rollback_checkpoint;

pub fn summarize_scheduled_rollback_notices_for_record(
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

pub fn derive_default_rollback_records_from_notice(
    items: &[rollback_checkpoint::ScheduledRollbackNotice],
) -> Vec<DefaultRollbackRecord> {
    let mut out: Vec<DefaultRollbackRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_default_rollback_record(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_default_rollback_record(item: &DefaultRollbackRecord) -> String {
    rollback_checkpoint::classify_rollback_checkpoint(item.weighted_value())
}
