use super::*;

use std::collections::BTreeMap;

use crate::suspension::pending_suspension_metric as suspension_pending_suspension_metric;

pub fn summarize_manual_suspension_quotas_for_estimate(
    items: &[suspension_pending_suspension_metric::ManualSuspensionQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_manual_migration_estimates_from_quota(
    items: &[suspension_pending_suspension_metric::ManualSuspensionQuota],
) -> Vec<ManualMigrationEstimate> {
    let mut out: Vec<ManualMigrationEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_migration_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_manual_migration_estimate(item: &ManualMigrationEstimate) -> String {
    suspension_pending_suspension_metric::classify_scheduled_suspension_metric(item.adjusted_exposure())
}
