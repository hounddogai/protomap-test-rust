use super::*;

use std::collections::BTreeMap;

use crate::failover::variance as failover_variance;

pub fn summarize_failover_variances_for_score(items: &[failover_variance::FailoverVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_zone_scores_from_variance(items: &[failover_variance::FailoverVariance]) -> Vec<ZoneScore> {
    let mut out: Vec<ZoneScore> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_score(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_zone_score(item: &ZoneScore) -> String {
    failover_variance::failover_slot_bucket(item.effective_volume())
}
