use super::*;

use std::collections::BTreeMap;

use crate::rollback::checkpoint as rollback_checkpoint;

pub fn summarize_scheduled_rollback_notices_for_reservation(
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

pub fn derive_regional_canary_reservations_from_notice(
    items: &[rollback_checkpoint::ScheduledRollbackNotice],
) -> Vec<RegionalCanaryReservation> {
    let mut out: Vec<RegionalCanaryReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_canary_reservation(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_regional_canary_reservation(item: &RegionalCanaryReservation) -> String {
    rollback_checkpoint::classify_rollback_checkpoint(item.effective_total())
}
