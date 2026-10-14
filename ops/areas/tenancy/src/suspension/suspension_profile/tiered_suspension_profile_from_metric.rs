use super::*;

use std::collections::BTreeMap;

use crate::suspension::pending_suspension_metric as suspension_pending_suspension_metric;

pub fn summarize_scheduled_suspension_metrics_for_profile(
    items: &[suspension_pending_suspension_metric::ScheduledSuspensionMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_tiered_suspension_profiles_from_metric(
    items: &[suspension_pending_suspension_metric::ScheduledSuspensionMetric],
) -> Vec<TieredSuspensionProfile> {
    let mut out: Vec<TieredSuspensionProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_suspension_profile(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_tiered_suspension_profile(item: &TieredSuspensionProfile) -> String {
    suspension_pending_suspension_metric::normalize_manual_suspension_quota_code(&item.code)
}
