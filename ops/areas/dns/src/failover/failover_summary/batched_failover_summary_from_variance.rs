use super::*;

use std::collections::BTreeMap;

use crate::health_check::entry as health_check_entry;

pub fn summarize_tiered_health_check_variances_for_summary(
    items: &[health_check_entry::TieredHealthCheckVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_batched_failover_summaries_from_variance(
    items: &[health_check_entry::TieredHealthCheckVariance],
) -> Vec<BatchedFailoverSummary> {
    let mut out: Vec<BatchedFailoverSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_failover_summary(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_batched_failover_summary(item: &BatchedFailoverSummary) -> String {
    health_check_entry::normalize_tiered_health_check_variance_code(&item.code)
}
