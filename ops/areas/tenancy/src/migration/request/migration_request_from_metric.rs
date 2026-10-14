use super::*;

use std::collections::BTreeMap;

use crate::suspension::suspension_metric as suspension_suspension_metric;

pub fn summarize_capped_suspension_metrics_for_request(
    items: &[suspension_suspension_metric::CappedSuspensionMetric],
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

pub fn derive_migration_requests_from_metric(
    items: &[suspension_suspension_metric::CappedSuspensionMetric],
) -> Vec<MigrationRequest> {
    let mut out: Vec<MigrationRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_request(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_migration_request(item: &MigrationRequest) -> f64 {
    suspension_suspension_metric::capped_suspension_metric_weight(9, true)
}
