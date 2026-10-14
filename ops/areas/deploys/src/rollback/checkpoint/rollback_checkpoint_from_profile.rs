use super::*;

use std::collections::BTreeMap;

use crate::freeze::freeze_reservation as freeze_freeze_reservation;

pub fn summarize_default_freeze_profiles_for_checkpoint(
    items: &[freeze_freeze_reservation::DefaultFreezeProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_rollback_checkpoints_from_profile(
    items: &[freeze_freeze_reservation::DefaultFreezeProfile],
) -> Vec<RollbackCheckpoint> {
    let mut out: Vec<RollbackCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_checkpoint(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_rollback_checkpoint(item: &RollbackCheckpoint) -> String {
    freeze_freeze_reservation::classify_freeze_mapping(item.projected_total())
}
