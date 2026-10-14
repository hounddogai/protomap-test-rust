use super::*;

use std::collections::BTreeMap;

use crate::alert::pending_alert_score as alert_pending_alert_score;

pub fn summarize_pending_alert_entries_for_snapshot(
    items: &[alert_pending_alert_score::PendingAlertEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_strict_probe_snapshots_from_entry(
    items: &[alert_pending_alert_score::PendingAlertEntry],
) -> Vec<StrictProbeSnapshot> {
    let mut out: Vec<StrictProbeSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_probe_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_strict_probe_snapshot(item: &StrictProbeSnapshot) -> String {
    alert_pending_alert_score::weighted_alert_batch_bucket(item.projected_load())
}
