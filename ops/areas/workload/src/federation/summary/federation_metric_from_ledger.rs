use super::*;

use std::collections::BTreeMap;

use crate::federation::cycle as federation_cycle;

pub fn summarize_batched_federation_ledgers_for_metric(
    items: &[federation_cycle::BatchedFederationLedger],
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

pub fn derive_federation_metrics_from_ledger(
    items: &[federation_cycle::BatchedFederationLedger],
) -> Vec<FederationMetric> {
    let mut out: Vec<FederationMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_metric(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_federation_metric(item: &FederationMetric) -> f64 {
    federation_cycle::scheduled_federation_reservation_weight(8, true)
}
