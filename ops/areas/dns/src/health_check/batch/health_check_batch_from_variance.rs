use super::*;

use std::collections::BTreeMap;

use crate::health_check::score as health_check_score;

pub fn summarize_primary_health_check_variances_for_batch(
    items: &[health_check_score::PrimaryHealthCheckVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_health_check_batches_from_variance(
    items: &[health_check_score::PrimaryHealthCheckVariance],
) -> Vec<HealthCheckBatch> {
    let mut out: Vec<HealthCheckBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_batch(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_health_check_batch(item: &HealthCheckBatch) -> f64 {
    health_check_score::primary_health_check_variance_weight(6, true)
}
